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
use anyhow::Context;
use anyhow::Result;

use crate::TeaclaveFileCryptoInfo;
use crate::TeaclaveFileRootKey128;
use protected_fs::ProtectedFile;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[macro_export]
macro_rules! hashmap {
    ($( $key: expr => $value: expr,)+) => { hashmap!($($key => $value),+) };
    ($( $key: expr => $value: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $value); )*
         map
    }}
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Clone, Debug, Default)]
pub struct TeaclaveWorkerInputFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFileRootKey128,
}

#[derive(Clone, Debug, Default)]
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
        log::debug!("path: {:?}", self.path);
        log::debug!("key: {:?}", self.crypto_info.key);
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
        // let crypto = TeaclaveFileRootKey128::default();
        let crypto = TeaclaveFileRootKey128::new(&[0; 16]).unwrap();
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
    crypto_info: TeaclaveFileCryptoInfo,
    dst: impl AsRef<std::path::Path>,
) -> anyhow::Result<TeaclaveWorkerInputFileInfo> {
    log::debug!("from: {:?}, to: {:?}", path.as_ref(), dst.as_ref());
    #[cfg(not(feature = "mesalock_sgx"))]
    use std::fs;
    #[cfg(feature = "mesalock_sgx")]
    use std::untrusted::fs;
    let plain_text = match crypto_info {
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
            fs::copy(path, dst.as_ref())?;
            let dst = dst.as_ref().to_owned();
            return Ok(TeaclaveWorkerInputFileInfo::new(dst, crypto));
        }
        TeaclaveFileCryptoInfo::Raw => read_all_bytes(path)?,
    };
    TeaclaveWorkerInputFileInfo::create_with_bytes(dst.as_ref(), &plain_text)
}

#[derive(Debug, Default)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgumentValue {
    inner: String,
}

impl ArgumentValue {
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    pub fn inner(&self) -> &String {
        &self.inner
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn as_usize(&self) -> Result<usize> {
        usize::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_u32(&self) -> Result<u32> {
        u32::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_f32(&self) -> Result<f32> {
        f32::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_f64(&self) -> Result<f64> {
        f64::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }

    pub fn as_u8(&self) -> Result<u8> {
        u8::from_str(&self.inner).with_context(|| format!("cannot parse {}", self.inner))
    }
}

impl std::fmt::Display for ArgumentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FunctionArguments {
    #[serde(flatten)]
    pub inner: HashMap<String, ArgumentValue>,
}

impl<S: core::default::Default + std::hash::BuildHasher> From<FunctionArguments>
    for HashMap<String, String, S>
{
    fn from(arguments: FunctionArguments) -> Self {
        arguments
            .inner()
            .iter()
            .map(|(k, v)| (k.to_owned(), v.as_str().to_owned()))
            .collect()
    }
}

impl From<HashMap<String, String>> for FunctionArguments {
    fn from(map: HashMap<String, String>) -> Self {
        FunctionArguments::from_map(&map)
    }
}

impl FunctionArguments {
    pub fn from_map<K, V>(input: &HashMap<K, V>) -> Self
    where
        K: std::string::ToString,
        V: std::string::ToString,
    {
        let inner = input.iter().fold(HashMap::new(), |mut acc, (k, v)| {
            acc.insert(k.to_string(), ArgumentValue::new(v.to_string()));
            acc
        });

        Self { inner }
    }

    pub fn inner(&self) -> &HashMap<String, ArgumentValue> {
        &self.inner
    }

    pub fn get(&self, key: &str) -> anyhow::Result<&ArgumentValue> {
        self.inner
            .get(key)
            .with_context(|| format!("key not found: {}", key))
    }

    pub fn into_vec(self) -> Vec<String> {
        let mut vector = Vec::new();

        self.inner.into_iter().for_each(|(k, v)| {
            vector.push(k);
            vector.push(v.to_string());
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
    pub function_args: FunctionArguments,
    pub input_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerInputFileInfo>,
    pub output_files: TeaclaveWorkerFileRegistry<TeaclaveWorkerOutputFileInfo>,
}

#[derive(Default)]
pub struct WorkerInvocationResult {
    pub return_value: Vec<u8>,
    pub output_file_hash: HashMap<String, String>,
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
