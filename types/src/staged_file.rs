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

use crate::TeaclaveFile128Key;

use std::collections::HashMap;
#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;
use std::io::{self, Read, Write};
use std::prelude::v1::*;
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

use protected_fs::ProtectedFile;

#[derive(Clone, Debug, Default)]
pub struct StagedFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFile128Key,
}

impl StagedFileInfo {
    pub fn new(path: impl AsRef<std::path::Path>, crypto_info: TeaclaveFile128Key) -> Self {
        StagedFileInfo {
            path: path.as_ref().into(),
            crypto_info,
        }
    }

    pub fn create_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        log::debug!("Open Protected File: {:?}", self.path);
        let f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    pub fn create_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
        log::debug!("Create Protected File: {:?}", self.path);
        let f = ProtectedFile::create_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    #[cfg(test_mode)]
    pub fn create_with_plaintext_file(
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<StagedFileInfo> {
        let bytes = read_all_bytes(path.as_ref())?;
        let dst = path.as_ref().with_extension("test_enc");
        Self::create_with_bytes(dst, &bytes)
    }

    #[cfg(test_mode)]
    pub fn get_plaintext(&self) -> anyhow::Result<Vec<u8>> {
        let mut content = Vec::new();
        let mut f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        f.read_to_end(&mut content)?;
        Ok(content)
    }

    pub fn create_with_bytes(
        path: impl AsRef<std::path::Path>,
        bytes: &[u8],
    ) -> anyhow::Result<StagedFileInfo> {
        let crypto = TeaclaveFile128Key::random();
        let mut f = ProtectedFile::create_ex(&path, &crypto.key)?;
        f.write_all(bytes)?;
        Ok(Self::new(path.as_ref(), crypto))
    }
}

pub fn read_all_bytes(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut content)?;
    Ok(content)
}

#[derive(Debug, Default)]
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
}

impl std::iter::FromIterator<(String, StagedFileInfo)> for StagedFiles {
    fn from_iter<T: IntoIterator<Item = (String, StagedFileInfo)>>(iter: T) -> Self {
        StagedFiles {
            entries: HashMap::from_iter(iter),
        }
    }
}
