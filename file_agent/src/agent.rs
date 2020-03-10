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

use futures::future::join_all;
use futures::TryFutureExt;
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::codec;
use url::Url;

async fn download_remote_input_to_file(
    presigned_url: Url,
    dest: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut download = reqwest::get(presigned_url.as_str())
        .await?
        .error_for_status()?;

    let mut outfile = tokio::fs::File::create(dest).await?;

    while let Some(chunk) = download.chunk().await? {
        outfile.write(&chunk).await?;
    }

    // Must flush tokio::io::BufWriter manually.
    // It will *not* flush itself automatically when dropped.
    outfile.flush().await?;

    Ok(())
}

async fn copy_file(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    tokio::fs::copy(src, dst).await?;
    Ok(())
}

async fn upload_output_file_to_remote(
    src: impl AsRef<std::path::Path>,
    presigned_url: Url,
) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(&src)?;
    let file_len = metadata.len();

    let stream = tokio::fs::File::open(src.as_ref().to_path_buf())
        .map_ok(|file| codec::FramedRead::new(file, codec::BytesCodec::new()))
        .try_flatten_stream();

    let body = reqwest::Body::wrap_stream(stream);

    let client = reqwest::Client::new();
    let res = client
        .put(presigned_url.as_str())
        .header(reqwest::header::CONTENT_TYPE, "application/x-binary")
        .header(reqwest::header::CONTENT_LENGTH, file_len.to_string())
        .body(body)
        .send()
        .await?;
    match res.status() {
        http::StatusCode::OK => Ok(()),
        status => anyhow::bail!("{}", status),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleFileInfo {
    local: PathBuf,
    remote: url::Url,
}
impl HandleFileInfo {
    pub fn new(local: PathBuf, remote: url::Url) -> Self {
        HandleFileInfo { local, remote }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HandleFileCommand {
    Download,
    Upload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAgentRequest {
    pub cmd: HandleFileCommand,
    pub info: Vec<HandleFileInfo>,
}

impl FileAgentRequest {
    pub fn new(cmd: HandleFileCommand, info: Vec<HandleFileInfo>) -> Self {
        FileAgentRequest { cmd, info }
    }
}

async fn handle_download(info: HandleFileInfo) -> anyhow::Result<()> {
    anyhow::ensure!(
        info.local.exists() == false,
        "[Download] Dest local file: {:?} already exists.",
        info.local
    );
    let dst = info.local;

    match info.remote.scheme() {
        "https" | "http" => {
            download_remote_input_to_file(info.remote, dst).await?;
        }
        "file" => {
            let src = PathBuf::from(info.remote.path());
            anyhow::ensure!(
                src.exists(),
                "[Download] Src local file: {:?} doesn't exist.",
                src
            );
            copy_file(src, dst).await?;
        }
        _ => anyhow::bail!("Scheme not supported"),
    }
    Ok(())
}

async fn handle_upload(info: HandleFileInfo) -> anyhow::Result<()> {
    anyhow::ensure!(
        info.local.exists(),
        "[Upload] Src local file: {:?} doesn't exist.",
        info.local
    );
    let src = info.local;

    match info.remote.scheme() {
        "https" | "http" => {
            upload_output_file_to_remote(src, info.remote).await?;
        }
        "file" => {
            let dst = PathBuf::from(info.remote.path());
            anyhow::ensure!(
                dst.exists(),
                "[Download] Dest local file: {:?} doesn't exist.",
                dst
            );
            copy_file(src, dst).await?;
        }
        _ => anyhow::bail!("Scheme not supported"),
    }
    Ok(())
}

fn handle_file_request(bytes: &[u8]) -> anyhow::Result<()> {
    let req: FileAgentRequest = serde_json::from_slice(bytes)?;
    let results = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()?
        .block_on(async {
            match req.cmd {
                HandleFileCommand::Download => {
                    let futures: Vec<_> = req
                        .info
                        .into_iter()
                        .map(|info| tokio::spawn(async { handle_download(info).await }))
                        .collect();
                    join_all(futures).await
                }
                HandleFileCommand::Upload => {
                    let futures: Vec<_> = req
                        .info
                        .into_iter()
                        .map(|info| tokio::spawn(async { handle_upload(info).await }))
                        .collect();
                    join_all(futures).await
                }
            }
        });

    let (task_results, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
    if errs.len() > 0 {
        anyhow::bail!("Spawned task join error!");
    }
    debug!("{:?}", task_results);
    anyhow::ensure!(
        task_results.into_iter().all(|x| x.unwrap().is_ok()),
        "Some handle file task failed"
    );
    Ok(())
}

#[no_mangle]
pub extern "C" fn ocall_handle_file_request(in_buf: *const u8, in_len: usize) -> u32 {
    let input_buf: &[u8] = unsafe { std::slice::from_raw_parts(in_buf, in_len) };
    match handle_file_request(input_buf) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use url::Url;

    #[test]
    fn test_file_url() {
        let s = "file:///tmp/abc.txt";
        let url = Url::parse(s).unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), "/tmp/abc.txt");
    }

    #[test]
    fn test_get_single_file() {
        let s = "http://localhost:6789/fixtures/functions/mesapy/input.txt";
        let url = Url::parse(s).unwrap();
        let dest = PathBuf::from("/tmp/input_test.txt");

        let info = HandleFileInfo::new(dest.clone(), url);
        let req = FileAgentRequest::new(HandleFileCommand::Download, vec![info]);

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_file(dest).unwrap();
    }

    #[test]
    fn test_put_single_file() {
        let src = PathBuf::from("/tmp/output_test.txt");
        {
            let mut file = std::fs::File::create(&src).unwrap();
            file.write_all(b"Hello Teaclave Results!").unwrap();
        }

        let s = "http://localhost:6789/fixtures/functions/mesapy/result.txt";
        let url = Url::parse(s).unwrap();

        let info = HandleFileInfo::new(src.clone(), url);
        let req = FileAgentRequest::new(HandleFileCommand::Upload, vec![info]);

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();
    }
}
