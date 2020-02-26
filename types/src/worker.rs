use std::collections::HashMap;
use std::collections::HashSet;
use std::format;
use std::io::{self, Read, Write};
use std::prelude::v1::*;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::fs::File;

#[cfg(not(feature = "mesalock_sgx"))]
use std::fs::File;

use anyhow;

use crate::TeaclaveFileCryptoInfo;
use crate::TeaclaveFileRootKey128;
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

#[derive(Debug)]
pub struct InputData {
    pub path: std::path::PathBuf,
    pub hash: String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}
#[derive(Debug)]
pub struct OutputData {
    pub path: std::path::PathBuf,
    pub hash: String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(Clone, Debug)]
pub struct TeaclaveWorkerInputFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFileRootKey128,
}

#[derive(Clone, Debug)]
pub struct TeaclaveWorkerOutputFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFileRootKey128,
}

impl std::convert::From<TeaclaveWorkerOutputFileInfo> for TeaclaveWorkerInputFileInfo {
    fn from(info: TeaclaveWorkerOutputFileInfo) -> Self {
        TeaclaveWorkerInputFileInfo {
            path: info.path,
            crypto_info: info.crypto_info,
        }
    }
}

impl TeaclaveWorkerInputFileInfo {
    pub fn new(
        path: impl std::convert::Into<std::path::PathBuf>,
        crypto_info: TeaclaveFileRootKey128,
    ) -> Self {
        TeaclaveWorkerInputFileInfo {
            path: path.into(),
            crypto_info,
        }
    }

    pub fn get_readable_io(&self) -> anyhow::Result<Box<dyn io::Read>> {
        let f = ProtectedFile::open_ex(&self.path, &self.crypto_info.key)?;
        Ok(Box::new(f))
    }

    #[cfg(test_mode)]
    pub fn create_with_plaintext_file(
        path: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<TeaclaveWorkerInputFileInfo> {
        let bytes = read_all_bytes(path.as_ref())?;
        let dst = path.as_ref().with_extension("enc");
        Self::create_with_bytes(dst, &bytes)
    }

    pub fn create_with_bytes(
        path: impl AsRef<std::path::Path>,
        bytes: &[u8],
    ) -> anyhow::Result<TeaclaveWorkerInputFileInfo> {
        let crypto = TeaclaveFileRootKey128::default();
        let mut f = ProtectedFile::create_ex(&path, &crypto.key)?;
        f.write_all(bytes)?;
        Ok(Self::new(path.as_ref(), crypto))
    }
}

impl TeaclaveWorkerOutputFileInfo {
    pub fn new(
        path: impl std::convert::Into<std::path::PathBuf>,
        crypto_info: TeaclaveFileRootKey128,
    ) -> Self {
        TeaclaveWorkerOutputFileInfo {
            path: path.into(),
            crypto_info,
        }
    }

    pub fn get_writable_io(&self) -> anyhow::Result<Box<dyn io::Write>> {
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
    src: InputData,
    dst: &str,
) -> anyhow::Result<TeaclaveWorkerInputFileInfo> {
    let path = src.path;
    let plain_text = match src.crypto_info {
        TeaclaveFileCryptoInfo::AesGcm128(crypto) => {
            let mut bytes = read_all_bytes(path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        TeaclaveFileCryptoInfo::AesGcm256(crypto) => {
            let mut bytes = read_all_bytes(path)?;
            crypto.decrypt(&mut bytes)?;
            bytes
        }
        TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(crypto) => {
            return Ok(TeaclaveWorkerInputFileInfo::new(path, crypto))
        }
    };
    TeaclaveWorkerInputFileInfo::create_with_bytes(dst, &plain_text)
}

#[derive(Debug)]
pub struct TeaclaveWorkerFileRegistry<T> {
    pub entries: HashMap<String, T>,
}

impl<T> TeaclaveWorkerFileRegistry<T> {
    pub fn new(entries: HashMap<String, T>) -> Self {
        TeaclaveWorkerFileRegistry { entries }
    }
}

impl<U, V> std::convert::TryFrom<HashMap<String, U>> for TeaclaveWorkerFileRegistry<V>
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
        Ok(TeaclaveWorkerFileRegistry { entries: out_info })
    }
}

impl<U, V, S> std::convert::From<TeaclaveWorkerFileRegistry<U>> for HashMap<String, V, S>
where
    V: std::convert::From<U>,
    S: std::hash::BuildHasher + Default,
{
    fn from(reg: TeaclaveWorkerFileRegistry<U>) -> Self {
        let mut out_info: HashMap<String, V, S> = HashMap::default();
        reg.entries
            .into_iter()
            .for_each(|(fid, finfo): (String, U)| {
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
pub struct WorkerCapability {
    pub runtimes: HashSet<String>,
    pub functions: HashSet<String>,
}

#[derive(Debug)]
pub struct WorkerInvocation {
    pub runtime_name: String,
    pub executor_type: TeaclaveExecutorSelector, // "native" | "python"
    pub function_name: String,                   // "gbdt_training" | "mesapy" |
    pub function_payload: String,
    pub function_args: TeaclaveFunctionArguments,
    pub input_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
    pub output_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
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
