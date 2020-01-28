#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow;
use serde::{Deserialize, Serialize};
use std::format;

const AES_GCM_256_KEY_LENGTH: usize = 32;
const AES_GCM_256_IV_LENGTH: usize = 12;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AesGcm256CryptoInfo {
    pub key: [u8; 32],
    pub iv: [u8; 12],
}

impl AesGcm256CryptoInfo {
    pub fn try_new(key: &[u8], iv: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            key.len() == AES_GCM_256_KEY_LENGTH,
            "Invalid key length for AesGcm256: {}",
            key.len()
        );
        anyhow::ensure!(
            iv.len() == AES_GCM_256_IV_LENGTH,
            "Invalid iv length for AesGcm256: {}",
            iv.len()
        );
        let mut info = AesGcm256CryptoInfo::default();
        info.key.copy_from_slice(key);
        info.iv.copy_from_slice(iv);
        Ok(info)
    }
}

const AES_GCM_128_KEY_LENGTH: usize = 16;
const AES_GCM_128_IV_LENGTH: usize = 12;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AesGcm128CryptoInfo {
    pub key: [u8; AES_GCM_128_KEY_LENGTH],
    pub iv: [u8; AES_GCM_128_IV_LENGTH],
}

impl AesGcm128CryptoInfo {
    pub fn try_new(key: &[u8], iv: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            key.len() == AES_GCM_128_KEY_LENGTH,
            "Invalid key length for AesGcm128: {}",
            key.len()
        );

        anyhow::ensure!(
            iv.len() == AES_GCM_128_IV_LENGTH,
            "Invalid iv length for AesGcm128: {}",
            iv.len()
        );

        let mut info = AesGcm128CryptoInfo::default();
        info.key.copy_from_slice(key);
        info.iv.copy_from_slice(iv);
        Ok(info)
    }
}

const TEACLAVE_FILE_ROOT_KEY_128_LENGTH: usize = 16;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TeaclaveFileRootKey128 {
    pub key: [u8; 16],
}

impl TeaclaveFileRootKey128 {
    pub fn try_new(key: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            key.len() == TEACLAVE_FILE_ROOT_KEY_128_LENGTH,
            "Invalid key length for teaclave_file_root_key_128: {}",
            key.len()
        );
        let mut info = TeaclaveFileRootKey128::default();
        info.key.copy_from_slice(key);
        Ok(info)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum TeaclaveFileCryptoInfo {
    AesGcm128(AesGcm128CryptoInfo),
    AesGcm256(AesGcm256CryptoInfo),
    TeaclaveFileRootKey128(TeaclaveFileRootKey128),
}
