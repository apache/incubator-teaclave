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
use tokio::io::AsyncWriteExt;
use tokio_util::codec;
use url::Url;

use std::path::{Component, Path, PathBuf};
use teaclave_types::{FileAgentRequest, HandleFileCommand, HandleFileInfo};

async fn download_remote_input_to_file(
    presigned_url: Url,
    dest: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut download = reqwest::get(presigned_url.as_str())
        .await?
        .error_for_status()?;

    let mut outfile = tokio::fs::File::create(dest).await?;

    while let Some(chunk) = download.chunk().await? {
        outfile.write_all(&chunk).await?;
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

async fn handle_download(
    info: HandleFileInfo,
    fusion_base: impl AsRef<Path>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !info.local.exists(),
        "[Download] Dest local file: {:?} already exists.",
        info.local
    );
    let dst = info.local;
    let remote = info.remote;

    match remote.scheme() {
        "https" | "http" => {
            download_remote_input_to_file(remote, dst).await?;
        }
        "file" => {
            let src = remote
                .to_file_path()
                .map_err(|e| anyhow::anyhow!("Cannot convert file:// to path: {:?}", e))?;
            anyhow::ensure!(
                src.exists(),
                "[Download] Src local file: {:?} doesn't exist.",
                src
            );
            copy_file(src, dst).await?;
        }
        "fusion" => {
            let path = remote
                .to_file_path()
                .map_err(|e| anyhow::anyhow!("Cannot convert fusion:// to path: {:?}", e))?;
            let components = path.components().collect::<Vec<_>>();
            anyhow::ensure!(
                (components[0] == Component::RootDir)
                    && (components[1] == Component::Normal("TEACLAVE_FUSION_BASE".as_ref())),
                "[Download] Fusion data format error: {:?}",
                components
            );

            let relative_path: PathBuf = components[2..].iter().collect();
            let src: PathBuf = fusion_base.as_ref().join(relative_path);

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

async fn handle_upload(info: HandleFileInfo, fusion_base: impl AsRef<Path>) -> anyhow::Result<()> {
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
            let dst = info
                .remote
                .to_file_path()
                .map_err(|e| anyhow::anyhow!("Cannot convert to path: {:?}", e))?;
            anyhow::ensure!(!dst.exists(), "[Upload] Dest local file: {:?} exist.", dst);
            copy_file(src, dst).await?;
        }
        "fusion" => {
            let path = info
                .remote
                .to_file_path()
                .map_err(|e| anyhow::anyhow!("Cannot convert fusion:// to path: {:?}", e))?;
            let components = path.components().collect::<Vec<_>>();
            anyhow::ensure!(
                (components[0] == Component::RootDir)
                    && (components[1] == Component::Normal("TEACLAVE_FUSION_BASE".as_ref())),
                "[Upload] Fusion data format error: {:?}",
                components
            );

            let relative_path: PathBuf = components[2..].iter().collect();
            let dst: PathBuf = fusion_base.as_ref().join(relative_path);

            anyhow::ensure!(
                !dst.exists(),
                "[Upload] Dest fusion file: {:?} exists.",
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
            let fusion_base = req.fusion_base.clone();
            match req.cmd {
                HandleFileCommand::Download => {
                    let futures: Vec<_> = req
                        .info
                        .into_iter()
                        .map(|info| {
                            let fusion_base = fusion_base.clone();
                            tokio::spawn(async { handle_download(info, fusion_base).await })
                        })
                        .collect();
                    join_all(futures).await
                }
                HandleFileCommand::Upload => {
                    let futures: Vec<_> = req
                        .info
                        .into_iter()
                        .map(|info| {
                            let fusion_base = fusion_base.clone();
                            tokio::spawn(async { handle_upload(info, fusion_base).await })
                        })
                        .collect();
                    join_all(futures).await
                }
            }
        });

    let (task_results, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

    debug!("{:?}, errs: {:?}", task_results, errs);
    if !errs.is_empty() {
        anyhow::bail!("Spawned task join error!");
    }
    anyhow::ensure!(
        task_results.into_iter().all(|x| x.unwrap().is_ok()),
        "Some handle file task failed"
    );
    Ok(())
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ocall_handle_file_request(in_buf: *const u8, in_len: u32) -> u32 {
    let input_buf: &[u8] = unsafe { std::slice::from_raw_parts(in_buf, in_len as usize) };
    match handle_file_request(input_buf) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use url::Url;

    #[test]
    fn test_file_url() {
        let url = Url::parse("file:///tmp/abc.txt").unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), "/tmp/abc.txt");

        let url = Url::parse("file:///countries/việt nam").unwrap();
        assert_eq!(url.path(), "/countries/vi%E1%BB%87t%20nam");
        let file_path = url.to_file_path().unwrap();
        assert_eq!(file_path, PathBuf::from("/countries/việt nam"));
    }

    #[test]
    fn test_get_single_file() {
        let s = "http://localhost:6789/fixtures/functions/mesapy/input.txt";
        let url = Url::parse(s).unwrap();
        let dest = PathBuf::from("/tmp/input_test.txt");

        let info = HandleFileInfo::new(&dest, &url);
        let req = FileAgentRequest::new(HandleFileCommand::Download, vec![info], "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_file(&dest).unwrap();
    }

    #[test]
    fn test_put_single_file() {
        let src = PathBuf::from("/tmp/output_single_test.txt");
        {
            let mut file = std::fs::File::create(&src).unwrap();
            file.write_all(b"Hello Teaclave Results!").unwrap();
        }

        let s = "http://localhost:6789/fixtures/functions/mesapy/result.txt";
        let url = Url::parse(s).unwrap();

        let info = HandleFileInfo::new(&src, &url);
        let req = FileAgentRequest::new(HandleFileCommand::Upload, vec![info], "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_file(&src).unwrap();
    }

    #[test]
    fn test_get_multiple_files() {
        let s = "http://localhost:6789/fixtures/functions/gbdt_training/train.txt";
        let url = Url::parse(s).unwrap();

        let base = PathBuf::from("/tmp/file_agent_test_base");
        let fnames = vec!["a.txt", "b.txt", "c.txt", "d.txt"];

        std::fs::create_dir_all(&base).unwrap();
        let info_list: Vec<_> = fnames
            .iter()
            .map(|fname| HandleFileInfo::new(base.join(fname), &url))
            .collect();
        let req = FileAgentRequest::new(HandleFileCommand::Download, info_list, "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn test_put_multiple_files() {
        let src = PathBuf::from("/tmp/output_multiple_test.txt");
        {
            let mut file = std::fs::File::create(&src).unwrap();
            file.write_all(b"Hello Teaclave Results!").unwrap();
        }

        let s = "http://localhost:6789/fixtures/functions/gbdt_training";
        let url = Url::parse(s).unwrap();

        let fnames = vec!["a.txt", "b.txt", "c.txt", "d.txt"];
        let info_list: Vec<_> = fnames
            .iter()
            .map(|fname| {
                let mut url = url.clone();
                url.path_segments_mut().unwrap().push(fname);
                HandleFileInfo::new(&src, &url)
            })
            .collect();

        let req = FileAgentRequest::new(HandleFileCommand::Upload, info_list, "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_file(&src).unwrap();
    }

    #[test]
    fn test_local_copy_file() {
        let base_str = "/tmp/file_agent_local_copy";
        let base = PathBuf::from(&base_str);
        std::fs::create_dir_all(&base).unwrap();

        let src = base.join("src.txt");
        {
            let mut file = std::fs::File::create(&src).unwrap();
            file.write_all(b"Hello Teaclave Results!").unwrap();
        }

        // test local upload
        let s = format!("file://{}/d1.txt", base_str);
        let url = Url::parse(&s).unwrap();

        let info = HandleFileInfo::new(&src, &url);
        let req = FileAgentRequest::new(HandleFileCommand::Upload, vec![info], "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        // test local download
        let dest = base.join("d2.txt");
        let info = HandleFileInfo::new(&dest, &url);
        let req = FileAgentRequest::new(HandleFileCommand::Download, vec![info], "");

        let bytes = serde_json::to_vec(&req).unwrap();
        handle_file_request(&bytes).unwrap();

        std::fs::remove_dir_all(&base).unwrap();
    }
}
