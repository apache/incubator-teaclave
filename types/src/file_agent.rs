use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::prelude::v1::*;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct HandleFileInfo {
    pub local: PathBuf,
    pub remote: url::Url,
}
impl HandleFileInfo {
    pub fn new(local: impl AsRef<std::path::Path>, remote: &url::Url) -> Self {
        HandleFileInfo {
            local: local.as_ref().to_owned(),
            remote: remote.to_owned(),
        }
    }
}
