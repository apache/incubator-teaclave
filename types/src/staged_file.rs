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

use teaclave_crypto::TeaclaveFile128Key;

use std::collections::HashMap;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;
use std::io::Read;
#[cfg(feature = "protected_fs")]
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

use crate::FileAuthTag;

#[derive(Clone, Debug, Default)]
pub struct StagedFileInfo {
    pub path: PathBuf,
    pub crypto_info: TeaclaveFile128Key,
    pub cmac: FileAuthTag,
}

impl StagedFileInfo {
    pub fn new(
        path: impl AsRef<Path>,
        crypto_info: TeaclaveFile128Key,
        cmac: impl Into<FileAuthTag>,
    ) -> Self {
        StagedFileInfo {
            path: path.as_ref().into(),
            crypto_info,
            cmac: cmac.into(),
        }
    }

    #[cfg(feature = "protected_fs")]
    pub fn create_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        use anyhow::Context;
        let f = protected_fs::ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        let tag = f
            .current_meta_gmac()
            .context("Failed to get gmac from protected file")?;
        anyhow::ensure!(self.cmac == tag, "Corrupted input file: {:?}", self.path);
        Ok(Box::new(f))
    }

    #[cfg(feature = "protected_fs")]
    pub fn create_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
        let f = protected_fs::ProtectedFile::create_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    #[cfg(feature = "protected_fs")]
    pub fn convert_file(
        &self,
        dst: impl AsRef<Path>,
        crypto: TeaclaveFile128Key,
    ) -> anyhow::Result<StagedFileInfo> {
        use anyhow::Context;
        let src_file = protected_fs::ProtectedFile::open_ex(&self.path, &self.crypto_info.key)
            .context("Convert: failed to open src file")?;
        let mut dest_file = protected_fs::ProtectedFile::create_ex(dst.as_ref(), &crypto.key)
            .context("Convert: failed to create dst file")?;

        let mut reader = BufReader::with_capacity(4096, src_file);
        loop {
            let buffer = reader.fill_buf()?;
            let rd_len = buffer.len();
            if rd_len == 0 {
                break;
            }
            let wt_len = dest_file.write(buffer)?;
            anyhow::ensure!(
                rd_len == wt_len,
                "Cannot fully write to dest file: Rd({:?}) != Wt({:?})",
                rd_len,
                wt_len
            );
            reader.consume(rd_len);
        }
        dest_file
            .flush()
            .context("Convert: dst_file flush failed")?;
        let tag = dest_file
            .current_meta_gmac()
            .context("Convert: cannot get dst_file gmac")?;
        Ok(StagedFileInfo::new(dst, crypto, tag))
    }

    #[cfg(all(test_mode, feature = "protected_fs"))]
    pub fn create_with_plaintext_file(path: impl AsRef<Path>) -> anyhow::Result<StagedFileInfo> {
        let bytes = read_all_bytes(path.as_ref())?;
        let dst = path.as_ref().with_extension("test_enc");
        Self::create_with_bytes(dst, &bytes)
    }

    #[cfg(all(test_mode, feature = "protected_fs"))]
    pub fn get_plaintext(&self) -> anyhow::Result<Vec<u8>> {
        let mut content = Vec::new();
        let mut f = protected_fs::ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        f.read_to_end(&mut content)?;
        Ok(content)
    }

    #[cfg(feature = "protected_fs")]
    pub fn create_with_bytes(
        path: impl AsRef<Path>,
        bytes: &[u8],
    ) -> anyhow::Result<StagedFileInfo> {
        let crypto = TeaclaveFile128Key::random();
        let mut f = protected_fs::ProtectedFile::create_ex(&path, &crypto.key)?;
        f.write_all(bytes)?;
        f.flush()?;
        let tag = f.current_meta_gmac()?;
        Ok(Self::new(path.as_ref(), crypto, tag))
    }
}

pub fn read_all_bytes(path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut content)?;
    Ok(content)
}

#[derive(Debug, Default, Clone)]
pub struct StagedFiles {
    entries: HashMap<String, StagedFileInfo>,
}

impl StagedFiles {
    pub fn new(entries: HashMap<String, StagedFileInfo>) -> Self {
        StagedFiles { entries }
    }

    pub fn get(&self, key: &str) -> Option<&StagedFileInfo> {
        self.entries.get(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::iter::FromIterator<(String, StagedFileInfo)> for StagedFiles {
    fn from_iter<T: IntoIterator<Item = (String, StagedFileInfo)>>(iter: T) -> Self {
        StagedFiles {
            entries: HashMap::from_iter(iter),
        }
    }
}
