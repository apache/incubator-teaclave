use std::collections::HashMap;
use std::format;
use std::io::{self, Read};
use std::prelude::v1::*;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;

use anyhow;

use crate::TeaclaveFileCryptoInfo;
use protected_fs::ProtectedFile;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

#[derive(Debug)]
pub enum TeaclaveExecutorSelector {
    Native,
    Python,
}

impl std::convert::TryFrom<&str> for TeaclaveExecutorSelector {
    type Error = anyhow::Error;

    fn try_from(selector: &str) -> anyhow::Result<Self> {
        let sel = match selector {
            "python" => TeaclaveExecutorSelector::Python,
            "native" => TeaclaveExecutorSelector::Native,
            _ => anyhow::bail!("Invalid executor selector: {}", selector),
        };
        Ok(sel)
    }
}

impl std::fmt::Display for TeaclaveExecutorSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TeaclaveExecutorSelector::Native => write!(f, "native"),
            TeaclaveExecutorSelector::Python => write!(f, "python"),
        }
    }
}

pub struct ReadBuffer {
    bytes: Vec<u8>,
    remaining: usize,
}

impl ReadBuffer {
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        let remaining = bytes.len();
        ReadBuffer { bytes, remaining }
    }
}

impl io::Read for ReadBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = std::cmp::min(buf.len(), self.remaining);
        let cur = self.bytes.len() - self.remaining;
        buf[..amt].copy_from_slice(&self.bytes[cur..cur + amt]);
        self.remaining -= amt;
        Ok(amt)
    }
}

#[derive(Clone, Debug)]
pub struct TeaclaveWorkerFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

impl TeaclaveWorkerFileInfo {
    pub fn new(
        path: impl std::convert::Into<std::path::PathBuf>,
        crypto_info: TeaclaveFileCryptoInfo,
    ) -> Self {
        TeaclaveWorkerFileInfo {
            path: path.into(),
            crypto_info,
        }
    }
}

pub fn read_all_bytes(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut content)?;
    Ok(content)
}

fn teaclave_file_with_bytes(path: &str, bytes: &[u8]) -> anyhow::Result<TeaclaveWorkerFileInfo> {
    let crypto_info = TeaclaveFileCryptoInfo::default();
    let file_info = TeaclaveWorkerFileInfo::new(path, crypto_info);
    let mut f = file_info.get_writable_io()?;
    f.write_all(bytes)?;
    Ok(file_info)
}

pub fn convert_plaintext_file(src: &str, dst: &str) -> anyhow::Result<TeaclaveWorkerFileInfo> {
    let bytes = read_all_bytes(src)?;
    teaclave_file_with_bytes(dst, &bytes)
}

pub fn convert_encrypted_file(
    src: TeaclaveWorkerFileInfo,
    dst: &str,
) -> anyhow::Result<TeaclaveWorkerFileInfo> {
    let plain_text = match &src.crypto_info {
        TeaclaveFileCryptoInfo::AesGcm128(crypto) => {
            let mut bytes = read_all_bytes(src.path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        TeaclaveFileCryptoInfo::AesGcm256(crypto) => {
            let mut bytes = read_all_bytes(src.path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(_) => return Ok(src),
    };
    teaclave_file_with_bytes(dst, &plain_text)
}

impl TeaclaveWorkerFileInfo {
    pub fn get_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        let readable: Box<dyn io::Read> = match &self.crypto_info {
            TeaclaveFileCryptoInfo::AesGcm128(crypto) => {
                let mut bytes = read_all_bytes(&self.path)?;
                crypto.decrypt(&mut bytes)?;
                Box::new(ReadBuffer::from_vec(bytes))
            }
            TeaclaveFileCryptoInfo::AesGcm256(crypto) => {
                let mut bytes = read_all_bytes(&self.path)?;
                crypto.decrypt(&mut bytes)?;
                Box::new(ReadBuffer::from_vec(bytes))
            }
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(crypto) => {
                let f = ProtectedFile::open_ex(&self.path, &crypto.key)?;
                Box::new(f)
            }
        };
        Ok(readable)
    }

    pub fn get_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
        let writable: Box<dyn io::Write> = match &self.crypto_info {
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(crypto) => {
                let f = ProtectedFile::create_ex(&self.path, &crypto.key)?;
                Box::new(f)
            }
            _ => anyhow::bail!("Output file encryption type not supported"),
        };
        Ok(writable)
    }
}

#[derive(Debug)]
pub struct TeaclaveWorkerFileRegistry {
    pub entries: HashMap<String, TeaclaveWorkerFileInfo>,
}

impl TeaclaveWorkerFileRegistry {
    pub fn new(entries: HashMap<String, TeaclaveWorkerFileInfo>) -> Self {
        TeaclaveWorkerFileRegistry { entries }
    }
}

impl<T> std::convert::TryFrom<HashMap<String, T>> for TeaclaveWorkerFileRegistry
where
    T: std::convert::TryInto<TeaclaveWorkerFileInfo, Error = anyhow::Error>,
{
    type Error = anyhow::Error;
    fn try_from(entries: HashMap<String, T>) -> anyhow::Result<Self> {
        let mut out_info: HashMap<String, TeaclaveWorkerFileInfo> = HashMap::new();
        entries
            .into_iter()
            .try_for_each(|(fid, finfo): (String, T)| -> anyhow::Result<()> {
                out_info.insert(fid, finfo.try_into()?);
                Ok(())
            })?;
        Ok(TeaclaveWorkerFileRegistry { entries: out_info })
    }
}

impl<T, S> std::convert::From<TeaclaveWorkerFileRegistry> for HashMap<String, T, S>
where
    T: std::convert::From<TeaclaveWorkerFileInfo>,
    S: std::hash::BuildHasher + Default,
{
    fn from(reg: TeaclaveWorkerFileRegistry) -> Self {
        let mut out_info: HashMap<String, T, S> = HashMap::default();
        reg.entries
            .into_iter()
            .for_each(|(fid, finfo): (String, TeaclaveWorkerFileInfo)| {
                out_info.insert(fid, finfo.into());
            });
        out_info
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TeaclaveFunctionArguments {
    pub args: HashMap<String, String>,
}

impl TeaclaveFunctionArguments {
    pub fn new<K, V>(input: &HashMap<K, V>) -> Self
    where
        K: std::string::ToString,
        V: std::string::ToString,
    {
        let args = input.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.to_string(), v.to_string());
            acc
        });

        TeaclaveFunctionArguments { args }
    }

    pub fn try_get<T: std::str::FromStr>(&self, key: &str) -> anyhow::Result<T> {
        self.args
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Cannot find function argument"))
            .and_then(|s| {
                s.parse::<T>()
                    .map_err(|_| anyhow::anyhow!("parse argument error"))
            })
    }

    pub fn into_vec(self) -> Vec<String> {
        let mut vector = Vec::new();
        self.args.into_iter().for_each(|(k, v)| {
            vector.push(k);
            vector.push(v);
        });
        vector
    }
}

#[derive(Debug)]
pub struct WorkerInvocation {
    pub runtime_name: String,
    pub executor_type: TeaclaveExecutorSelector, // "native" | "python"
    pub function_name: String,                   // "gbdt_training" | "mesapy" |
    pub function_payload: String,
    pub function_args: TeaclaveFunctionArguments,
    pub input_files: TeaclaveWorkerFileRegistry,
    pub output_files: TeaclaveWorkerFileRegistry,
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    //use crate::unit_tests;
    //use crate::unittest::*;

    pub fn run_tests() -> bool {
        //unit_tests!()
        true
    }
}
