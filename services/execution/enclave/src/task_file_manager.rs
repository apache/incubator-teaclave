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

use crate::ocall::handle_file_request;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::prelude::v1::*;
use std::untrusted::path::PathEx;
use teaclave_crypto::TeaclaveFile128Key;
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

pub(crate) struct TaskFileManager {
    inter_inputs: InterInputs,
    inter_outputs: InterOutputs,
    fusion_base: PathBuf,
}

struct InterInputs {
    inner: Vec<InterInput>,
}

struct InterOutputs {
    inner: Vec<InterOutput>,
}

pub(self) struct InterInput {
    pub(self) funiq_key: String,
    pub(self) file: FunctionInputFile,
    pub(self) download_path: PathBuf,
    pub(self) staged_path: PathBuf,
}

pub(self) struct InterOutput {
    pub(self) funiq_key: String,
    pub(self) file: FunctionOutputFile,
    pub(self) upload_path: PathBuf,
    pub(self) staged_info: StagedFileInfo,
}

impl TaskFileManager {
    pub(crate) fn new(
        inter_base: impl AsRef<Path>,
        fusion_base: impl AsRef<Path>,
        task_id: &Uuid,
        inputs: &FunctionInputFiles,
        outputs: &FunctionOutputFiles,
    ) -> Result<Self> {
        let cwd = Path::new(inter_base.as_ref()).join(task_id.to_string());
        let inputs_base = cwd.join("inputs");
        let outputs_base = cwd.join("outputs");

        let inter_inputs = InterInputs::new(&inputs_base, inputs.clone())?;
        let inter_outputs = InterOutputs::new(&outputs_base, outputs.clone())?;

        let tfmgr = TaskFileManager {
            inter_inputs,
            inter_outputs,
            fusion_base: fusion_base.as_ref().to_owned(),
        };

        Ok(tfmgr)
    }

    pub(crate) fn prepare_staged_inputs(&self) -> Result<StagedFiles> {
        self.inter_inputs.download(&self.fusion_base)?;
        self.inter_inputs.convert_to_staged_files()
    }

    pub(crate) fn prepare_staged_outputs(&self) -> Result<StagedFiles> {
        let staged_outputs = self.inter_outputs.generate_staged_files();
        Ok(staged_outputs)
    }

    pub(crate) fn upload_outputs(&self) -> Result<HashMap<String, FileAuthTag>> {
        let auth_tags = self.inter_outputs.convert_staged_files_for_upload()?;
        self.inter_outputs.upload(&self.fusion_base)?;
        Ok(auth_tags)
    }
}

impl InterInput {
    fn new(
        inter_base: impl AsRef<Path>,
        funiq_key: String,
        file: FunctionInputFile,
    ) -> Result<InterInput> {
        let download_path = make_intermediate_path(inter_base.as_ref(), &funiq_key, &file.url)?;
        let staged_path = make_staged_path(inter_base.as_ref(), &funiq_key, &file.url)?;

        Ok(InterInput {
            funiq_key,
            file,
            download_path,
            staged_path,
        })
    }

    fn to_staged_file_entry(&self) -> Result<(String, StagedFileInfo)> {
        let src = &self.download_path;
        let dst = &self.staged_path;
        let staged_file_info = match self.file.crypto_info {
            FileCrypto::TeaclaveFile128(crypto) => {
                std::untrusted::fs::soft_link(src, dst)?;
                StagedFileInfo::new(&src, crypto, self.file.cmac)
            }
            FileCrypto::AesGcm128(crypto) => {
                let mut bytes = read_all_bytes(src)?;
                let n = bytes.len();
                anyhow::ensure!(
                    n > FILE_AUTH_TAG_LENGTH,
                    "AesGcm128 File, invalid length: {:?}",
                    src
                );
                anyhow::ensure!(
                    self.file.cmac == bytes[n - FILE_AUTH_TAG_LENGTH..],
                    "AesGcm128 File, invalid tag: {:?}",
                    src
                );
                crypto.decrypt(&mut bytes)?;
                StagedFileInfo::create_with_bytes(dst, &bytes)?
            }
            FileCrypto::AesGcm256(crypto) => {
                let mut bytes = read_all_bytes(src)?;
                let n = bytes.len();
                anyhow::ensure!(
                    n > FILE_AUTH_TAG_LENGTH,
                    "AesGcm256 File, invalid length: {:?}",
                    src
                );
                anyhow::ensure!(
                    self.file.cmac == bytes[n - FILE_AUTH_TAG_LENGTH..],
                    "AesGcm256 File, invalid tag: {:?}",
                    src
                );
                crypto.decrypt(&mut bytes)?;
                StagedFileInfo::create_with_bytes(dst, &bytes)?
            }
            FileCrypto::Raw => {
                let bytes = read_all_bytes(src)?;
                StagedFileInfo::create_with_bytes(dst, &bytes)?
            }
        };
        Ok((self.funiq_key.clone(), staged_file_info))
    }
}

impl std::iter::FromIterator<InterInput> for InterInputs {
    fn from_iter<T: IntoIterator<Item = InterInput>>(iter: T) -> Self {
        InterInputs {
            inner: Vec::from_iter(iter),
        }
    }
}

impl InterInputs {
    pub fn new(input_base: impl AsRef<Path>, inputs: FunctionInputFiles) -> Result<InterInputs> {
        inputs
            .into_iter()
            .map(|(funiq_key, file)| InterInput::new(input_base.as_ref(), funiq_key, file))
            .collect()
    }

    pub(crate) fn download(&self, fusion_base: impl AsRef<Path>) -> Result<()> {
        let req_info = self.inner.iter().map(|inter_input| {
            HandleFileInfo::new(&inter_input.download_path, &inter_input.file.url)
        });
        let request =
            FileAgentRequest::new(HandleFileCommand::Download, req_info, fusion_base.as_ref());
        log::debug!("Ocall file download request: {:?}", request);
        handle_file_request(request)?;
        Ok(())
    }

    pub(crate) fn convert_to_staged_files(&self) -> Result<StagedFiles> {
        self.inner
            .iter()
            .map(|inter_file| inter_file.to_staged_file_entry())
            .collect()
    }
}

impl std::iter::FromIterator<InterOutput> for InterOutputs {
    fn from_iter<T: IntoIterator<Item = InterOutput>>(iter: T) -> Self {
        InterOutputs {
            inner: Vec::from_iter(iter),
        }
    }
}

impl InterOutput {
    pub fn new(
        inter_base: impl AsRef<Path>,
        funiq_key: String,
        file: FunctionOutputFile,
    ) -> Result<InterOutput> {
        let upload_path = make_intermediate_path(inter_base.as_ref(), &funiq_key, &file.url)?;
        let staged_path = make_staged_path(inter_base.as_ref(), &funiq_key, &file.url)?;
        let random_key = TeaclaveFile128Key::random();
        let staged_info = StagedFileInfo::new(&staged_path, random_key, FileAuthTag::default());

        Ok(InterOutput {
            funiq_key,
            file,
            upload_path,
            staged_info,
        })
    }

    fn convert_to_upload_file(&self) -> Result<FileAuthTag> {
        let dest = &self.upload_path;
        let cmac = self
            .staged_info
            .convert_for_uploading(dest, self.file.crypto_info.to_owned())?;
        Ok(cmac)
    }
}

impl InterOutputs {
    pub fn new(
        output_base: impl AsRef<Path>,
        outputs: FunctionOutputFiles,
    ) -> Result<InterOutputs> {
        outputs
            .into_iter()
            .map(|(funiq_key, file)| InterOutput::new(output_base.as_ref(), funiq_key, file))
            .collect()
    }

    pub fn generate_staged_files(&self) -> StagedFiles {
        self.inner
            .iter()
            .map(|inter_output| {
                (
                    inter_output.funiq_key.clone(),
                    inter_output.staged_info.clone(),
                )
            })
            .collect()
    }

    pub fn convert_staged_files_for_upload(&self) -> Result<HashMap<String, FileAuthTag>> {
        self.inner
            .iter()
            .map(|inter_output| {
                inter_output
                    .convert_to_upload_file()
                    .map(|cmac| (inter_output.funiq_key.clone(), cmac))
            })
            .collect()
    }

    pub(crate) fn upload(&self, fusion_base: impl AsRef<Path>) -> Result<()> {
        let req_info = self.inner.iter().map(|inter_output| {
            HandleFileInfo::new(&inter_output.upload_path, &inter_output.file.url)
        });
        let request =
            FileAgentRequest::new(HandleFileCommand::Upload, req_info, fusion_base.as_ref());
        log::debug!("Ocall file upload request: {:?}", request);
        handle_file_request(request)?;
        Ok(())
    }
}

// Staged file is put in $base_dir/${funiq_key}-staged/$original_name
fn make_staged_path(base: impl AsRef<Path>, funiq_key: &str, url: &Url) -> Result<PathBuf> {
    let url_path = url.path();
    let original_name = Path::new(url_path)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Cannot get filename from url: {:?}", url))?;

    let staged_dir = format!("{}-{}", funiq_key, "staged");
    let file_dir = base.as_ref().to_owned().join(&staged_dir);
    if !file_dir.exists() {
        std::untrusted::fs::create_dir_all(&file_dir)?;
    }
    let local_dest = file_dir.join(original_name);
    Ok(local_dest)
}

// Intermediate file is converted to $base_dir/${funiq_key}/$original_name
fn make_intermediate_path(base: impl AsRef<Path>, funiq_key: &str, url: &Url) -> Result<PathBuf> {
    let url_path = url.path();
    let original_name = Path::new(url_path)
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Cannot get filename from url: {:?}", url))?;

    let file_dir = base.as_ref().to_owned().join(funiq_key);
    if !file_dir.exists() {
        std::untrusted::fs::create_dir_all(&file_dir)?;
    }
    let local_dest = file_dir.join(original_name);
    Ok(local_dest)
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_crypto::*;
    use url::Url;

    pub fn test_input() {
        let key = [0; 16];
        let iv = [1; 12];
        let crypto = AesGcm128Key::new(&key, &iv).unwrap();
        let input_url =
            Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/train.aes_gcm_128")
                .unwrap();
        let tag = FileAuthTag::from_hex("592f1e607649d89ff2aa8a2841a57cad").unwrap();
        let input_file = FunctionInputFile::new(input_url, tag, crypto);

        let output_url =
            Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/result.aes_gcm_128")
                .unwrap();
        let output_file = FunctionOutputFile::new(output_url, crypto);

        let inputs = hashmap!("training_data" => input_file);
        let outputs = hashmap!("result" => output_file);
        let task_id = Uuid::new_v4();

        let file_mgr = TaskFileManager::new(
            "/tmp",
            "/tmp/fusion_base",
            &task_id,
            &inputs.into(),
            &outputs.into(),
        )
        .unwrap();

        let input_files = file_mgr.prepare_staged_inputs().unwrap();
        let output_files = file_mgr.prepare_staged_outputs().unwrap();
        // sin_file has random key1
        let sin_file = input_files.get("training_data").unwrap();
        // sout_file has random key2
        let sout_file = output_files.get("result").unwrap();
        // convert sin_file to sout_file to simulate the executor's behavior
        sin_file
            .convert_to_teaclave_file(&sout_file.path, sout_file.crypto_info)
            .unwrap();
        file_mgr.upload_outputs().unwrap();
    }
}
