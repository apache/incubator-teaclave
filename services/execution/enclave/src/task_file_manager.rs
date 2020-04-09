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
use teaclave_types::*;
use url::Url;

pub(crate) struct TaskFileManager {
    cwd: PathBuf,
    inputs: FunctionInputFiles,
    outputs: FunctionOutputFiles,
}

impl TaskFileManager {
    pub(crate) fn new(base: &str, task: &StagedTask) -> Result<Self> {
        let cwd = Path::new(base).join(task.task_id.to_string());
        if !cwd.exists() {
            std::untrusted::fs::create_dir_all(&cwd)?;
        }

        let tfmgr = TaskFileManager {
            cwd,
            inputs: task.input_data.clone(),
            outputs: task.output_data.clone(),
        };

        Ok(tfmgr)
    }

    fn make_input_download_path(&self, funiq_key: &str, url: &Url) -> Result<PathBuf> {
        let url_path = url.path();
        let original_name = Path::new(url_path)
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Cannot get filename from url: {:?}", url))?;
        let download_dir = self.cwd.join(funiq_key);
        if !download_dir.exists() {
            std::untrusted::fs::create_dir_all(&download_dir)?;
        }
        let local_dest = download_dir.join(original_name);
        Ok(local_dest)
    }

    fn make_local_converted_path(&self, file_unique_key: &str) -> PathBuf {
        let mut local_dest = self.cwd.join(file_unique_key);
        local_dest.set_extension("converted");
        local_dest
    }

    fn make_local_output_path(&self, file_unique_key: &str) -> PathBuf {
        let mut local_dest = self.cwd.join(file_unique_key);
        local_dest.set_extension("out");
        local_dest
    }

    pub(crate) fn download_inputs(&self) -> Result<()> {
        let mut req_info = Vec::new();
        for (fname, finfo) in self.inputs.iter() {
            let local_dest = self.make_input_download_path(fname, &finfo.url)?;
            req_info.push(HandleFileInfo::new(local_dest, &finfo.url));
        }

        let request = FileAgentRequest::new(HandleFileCommand::Download, req_info);
        log::info!("Ocall file download request: {:?}", request);
        handle_file_request(request)?;
        Ok(())
    }

    pub(crate) fn convert_downloaded_inputs(&self) -> Result<StagedFiles> {
        let mut files: HashMap<String, StagedFileInfo> = HashMap::new();
        for (fkey, finfo) in self.inputs.iter() {
            let src = self.make_input_download_path(fkey, &finfo.url)?;
            let staged_file_info = match finfo.crypto_info {
                FileCrypto::TeaclaveFile128(crypto) => StagedFileInfo::new(&src, crypto),
                FileCrypto::AesGcm128(crypto) => {
                    let dst = self.make_local_converted_path(fkey);
                    let mut bytes = read_all_bytes(&src)?;
                    crypto.decrypt(&mut bytes)?;
                    StagedFileInfo::create_with_bytes(&dst, &bytes)?
                }
                FileCrypto::AesGcm256(crypto) => {
                    let dst = self.make_local_converted_path(fkey);
                    let mut bytes = read_all_bytes(&src)?;
                    crypto.decrypt(&mut bytes)?;
                    StagedFileInfo::create_with_bytes(&dst, &bytes)?
                }
                FileCrypto::Raw => {
                    let dst = self.make_local_converted_path(fkey);
                    let bytes = read_all_bytes(&src)?;
                    StagedFileInfo::create_with_bytes(&dst, &bytes)?
                }
            };

            files.insert(fkey.to_string(), staged_file_info);
        }
        Ok(StagedFiles::new(files))
    }

    pub(crate) fn prepare_staged_outputs(&self) -> Result<StagedFiles> {
        let mut files: HashMap<String, StagedFileInfo> = HashMap::new();
        for (fkey, finfo) in self.outputs.iter() {
            let dest = self.make_local_output_path(fkey);
            let crypto = match finfo.crypto_info {
                FileCrypto::TeaclaveFile128(crypto) => crypto,
                _ => anyhow::bail!("PrepareFile: unsupported output"),
            };
            files.insert(fkey.to_string(), StagedFileInfo::new(dest, crypto));
        }
        Ok(StagedFiles::new(files))
    }

    pub(crate) fn upload_outputs(&self) -> Result<()> {
        let req_info = self.outputs.iter().map(|(fkey, value)| {
            let local = self.make_local_output_path(fkey);
            HandleFileInfo::new(local, &value.url)
        });
        let request = FileAgentRequest::new(HandleFileCommand::Upload, req_info);
        log::info!("Ocall file upload request: {:?}", request);
        handle_file_request(request)?;
        Ok(())
    }
}
