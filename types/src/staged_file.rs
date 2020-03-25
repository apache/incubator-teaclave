// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::{FileCrypto, TeaclaveFile128Key};

use std::collections::HashMap;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;
use std::io::{self, Read, Write};
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

use protected_fs::ProtectedFile;

#[derive(Clone, Debug, Default)]
pub struct StagedInputFile {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFile128Key,
}

#[derive(Clone, Debug, Default)]
pub struct StagedOutputFile {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFile128Key,
}

impl std::convert::From<StagedOutputFile> for StagedInputFile {
    fn from(info: StagedOutputFile) -> Self {
        StagedInputFile {
            path: info.path,
            crypto_info: info.crypto_info,
        }
    }
}

impl StagedInputFile {
    pub fn new(
        path: impl std::convert::Into<std::path::PathBuf>,
        crypto_info: TeaclaveFile128Key,
    ) -> Self {
        StagedInputFile {
            path: path.into(),
            crypto_info,
        }
    }

    pub fn get_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        log::debug!("path: {:?}", self.path);
        log::debug!("key: {:?}", self.crypto_info.key);
        let f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    #[cfg(test_mode)]
    pub fn create_with_plaintext_file(
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<StagedInputFile> {
        let bytes = read_all_bytes(path.as_ref())?;
        let dst = path.as_ref().with_extension("enc");
        Self::create_with_bytes(dst, &bytes)
    }

    pub fn create_with_bytes(
        path: impl AsRef<std::path::Path>,
        bytes: &[u8],
    ) -> anyhow::Result<StagedInputFile> {
        let crypto = TeaclaveFile128Key::random();
        let mut f = ProtectedFile::create_ex(&path, &crypto.key)?;
        f.write_all(bytes)?;
        Ok(Self::new(path.as_ref(), crypto))
    }
}

impl StagedOutputFile {
    pub fn new(
        path: impl std::convert::Into<std::path::PathBuf>,
        crypto_info: TeaclaveFile128Key,
    ) -> Self {
        StagedOutputFile {
            path: path.into(),
            crypto_info,
        }
    }

    pub fn get_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
        log::debug!("path: {:?}", self.path);
        log::debug!("key: {:?}", self.crypto_info.key);
        let f = ProtectedFile::create_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    #[cfg(test_mode)]
    pub fn get_plaintext(&self) -> anyhow::Result<Vec<u8>> {
        let mut content = Vec::new();
        let mut f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        f.read_to_end(&mut content)?;
        Ok(content)
    }
}

pub fn read_all_bytes(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn convert_encrypted_input_file(
    path: impl AsRef<std::path::Path>,
    crypto_info: FileCrypto,
    dst: impl AsRef<std::path::Path>,
) -> anyhow::Result<StagedInputFile> {
    log::debug!("from: {:?}, to: {:?}", path.as_ref(), dst.as_ref());
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs;
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs;
    let plain_text = match crypto_info {
        FileCrypto::AesGcm128(crypto) => {
            let mut bytes = read_all_bytes(path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        FileCrypto::AesGcm256(crypto) => {
            let mut bytes = read_all_bytes(path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        FileCrypto::TeaclaveFile128(crypto) => {
            fs::copy(path, dst.as_ref())?;
            let dst = dst.as_ref().to_owned();
            return Ok(StagedInputFile::new(dst, crypto));
        }
        FileCrypto::Raw => read_all_bytes(path)?,
    };
    StagedInputFile::create_with_bytes(dst.as_ref(), &plain_text)
}

#[derive(Debug, Default)]
pub struct StagedFiles<T> {
    pub entries: HashMap<String, T>,
}

impl<T> StagedFiles<T> {
    pub fn new(entries: HashMap<String, T>) -> Self {
        StagedFiles { entries }
    }
}

impl<U, V> std::convert::TryFrom<HashMap<String, U>> for StagedFiles<V>
where
    U: std::convert::TryInto<V, Error = anyhow::Error>,
{
    type Error = anyhow::Error;
    fn try_from(entries: HashMap<String, U>) -> anyhow::Result<Self> {
        let mut out_info: HashMap<String, V> = HashMap::new();
        entries
            .into_iter()
            .try_for_each(|(fid, finfo): (String, U)| -> anyhow::Result<()> {
                out_info.insert(fid, finfo.try_into()?);
                Ok(())
            })?;
        Ok(StagedFiles { entries: out_info })
    }
}

impl<U, V, S> std::convert::From<StagedFiles<U>> for HashMap<String, V, S>
where
    V: std::convert::From<U>,
    S: std::hash::BuildHasher + Default,
{
    fn from(reg: StagedFiles<U>) -> Self {
        let mut out_info: HashMap<String, V, S> = HashMap::default();
        reg.entries
            .into_iter()
            .for_each(|(fid, finfo): (String, U)| {
                out_info.insert(fid, finfo.into());
            });
        out_info
    }
}
