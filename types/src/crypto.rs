#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow;
use rand::prelude::RngCore;
use ring;
use serde::{Deserialize, Serialize};
use std::format;

const AES_GCM_256_KEY_LENGTH: usize = 32;
const AES_GCM_256_IV_LENGTH: usize = 12;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AesGcm256CryptoInfo {
    pub key: [u8; AES_GCM_256_KEY_LENGTH],
    pub iv: [u8; AES_GCM_256_IV_LENGTH],
}

impl AesGcm256CryptoInfo {
    pub fn new(in_key: &[u8], in_iv: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            in_key.len() == AES_GCM_256_KEY_LENGTH,
            "Invalid key length for AesGcm256: {}",
            in_key.len()
        );
        anyhow::ensure!(
            in_iv.len() == AES_GCM_256_IV_LENGTH,
            "Invalid iv length for AesGcm256: {}",
            in_iv.len()
        );
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        key.copy_from_slice(in_key);
        iv.copy_from_slice(in_iv);
        Ok(AesGcm256CryptoInfo { key, iv })
    }

    pub fn decrypt(&self, in_out: &mut Vec<u8>) -> anyhow::Result<()> {
        let plaintext_len =
            aead_decrypt(&ring::aead::AES_256_GCM, in_out, &self.key, &self.iv)?.len();
        in_out.truncate(plaintext_len);
        Ok(())
    }

    pub fn encrypt(&self, in_out: &mut Vec<u8>) -> anyhow::Result<()> {
        aead_encrypt(&ring::aead::AES_128_GCM, in_out, &self.key, &self.iv)
    }
}

impl Default for AesGcm256CryptoInfo {
    fn default() -> Self {
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);
        AesGcm256CryptoInfo { key, iv }
    }
}

const AES_GCM_128_KEY_LENGTH: usize = 16;
const AES_GCM_128_IV_LENGTH: usize = 12;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AesGcm128CryptoInfo {
    pub key: [u8; AES_GCM_128_KEY_LENGTH],
    pub iv: [u8; AES_GCM_128_IV_LENGTH],
}

impl AesGcm128CryptoInfo {
    pub fn new(in_key: &[u8], in_iv: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            in_key.len() == AES_GCM_128_KEY_LENGTH,
            "Invalid key length for AesGcm128: {}",
            in_key.len()
        );

        anyhow::ensure!(
            in_iv.len() == AES_GCM_128_IV_LENGTH,
            "Invalid iv length for AesGcm128: {}",
            in_iv.len()
        );

        let mut key = [0u8; AES_GCM_128_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_128_IV_LENGTH];
        key.copy_from_slice(in_key);
        iv.copy_from_slice(in_iv);
        Ok(AesGcm128CryptoInfo { key, iv })
    }

    pub fn decrypt(&self, in_out: &mut Vec<u8>) -> anyhow::Result<()> {
        let plaintext_len =
            aead_decrypt(&ring::aead::AES_128_GCM, in_out, &self.key, &self.iv)?.len();
        in_out.truncate(plaintext_len);
        Ok(())
    }

    pub fn encrypt(&self, in_out: &mut Vec<u8>) -> anyhow::Result<()> {
        aead_encrypt(&ring::aead::AES_128_GCM, in_out, &self.key, &self.iv)
    }
}

impl Default for AesGcm128CryptoInfo {
    fn default() -> Self {
        let mut key = [0u8; AES_GCM_128_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_128_IV_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);
        AesGcm128CryptoInfo { key, iv }
    }
}

const TEACLAVE_FILE_ROOT_KEY_128_LENGTH: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeaclaveFileRootKey128 {
    pub key: [u8; TEACLAVE_FILE_ROOT_KEY_128_LENGTH],
}

impl TeaclaveFileRootKey128 {
    pub fn new(in_key: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            in_key.len() == TEACLAVE_FILE_ROOT_KEY_128_LENGTH,
            "Invalid key length for teaclave_file_root_key_128: {}",
            in_key.len()
        );
        let mut key = [0u8; TEACLAVE_FILE_ROOT_KEY_128_LENGTH];
        key.copy_from_slice(in_key);
        Ok(TeaclaveFileRootKey128 { key })
    }
}

impl Default for TeaclaveFileRootKey128 {
    fn default() -> Self {
        let mut key = [0u8; TEACLAVE_FILE_ROOT_KEY_128_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        TeaclaveFileRootKey128 { key }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TeaclaveFileCryptoInfo {
    AesGcm128(AesGcm128CryptoInfo),
    AesGcm256(AesGcm256CryptoInfo),
    TeaclaveFileRootKey128(TeaclaveFileRootKey128),
}

impl TeaclaveFileCryptoInfo {
    pub fn new(schema: &str, key: &[u8], iv: &[u8]) -> anyhow::Result<Self> {
        let info = match schema {
            "aes_gcm_128" => {
                let info = AesGcm128CryptoInfo::new(key, iv)?;
                TeaclaveFileCryptoInfo::AesGcm128(info)
            }
            "aes_gcm_256" => {
                let info = AesGcm256CryptoInfo::new(key, iv)?;
                TeaclaveFileCryptoInfo::AesGcm256(info)
            }
            "teaclave_file_root_key_128" => {
                anyhow::ensure!(
                    iv.is_empty(),
                    "IV is not empty for teaclave_file_root_key_128"
                );
                let info = TeaclaveFileRootKey128::new(key)?;
                TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(info)
            }
            _ => anyhow::bail!("Invalid crypto schema: {}", schema),
        };
        Ok(info)
    }

    pub fn schema(&self) -> String {
        match self {
            TeaclaveFileCryptoInfo::AesGcm128(_) => "aes_gcm_128".to_string(),
            TeaclaveFileCryptoInfo::AesGcm256(_) => "aes_gcm_256".to_string(),
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(_) => {
                "teaclave_file_root_key_128".to_string()
            }
        }
    }

    pub fn key_iv(&self) -> (Vec<u8>, Vec<u8>) {
        match self {
            TeaclaveFileCryptoInfo::AesGcm128(crypto) => (crypto.key.to_vec(), crypto.iv.to_vec()),
            TeaclaveFileCryptoInfo::AesGcm256(crypto) => (crypto.key.to_vec(), crypto.iv.to_vec()),
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(crypto) => {
                (crypto.key.to_vec(), Vec::new())
            }
        }
    }
}

impl Default for TeaclaveFileCryptoInfo {
    fn default() -> Self {
        TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(TeaclaveFileRootKey128::default())
    }
}

fn make_teaclave_aad() -> ring::aead::Aad<[u8; 8]> {
    let bytes = [0u8; 8];
    ring::aead::Aad::from(bytes)
}

pub fn aead_decrypt<'a>(
    alg: &'static ring::aead::Algorithm,
    in_out: &'a mut [u8],
    key: &[u8],
    iv: &[u8],
) -> anyhow::Result<&'a mut [u8]> {
    let key = ring::aead::UnboundKey::new(alg, key)
        .map_err(|_| anyhow::anyhow!("Aead unbound key init error"))?;
    let nonce = ring::aead::Nonce::try_assume_unique_for_key(iv)
        .map_err(|_| anyhow::anyhow!("Aead iv init error"))?;
    let aad = make_teaclave_aad();

    let dec_key = ring::aead::LessSafeKey::new(key);
    let slice = dec_key
        .open_in_place(nonce, aad, in_out)
        .map_err(|_| anyhow::anyhow!("Aead open_in_place error"))?;
    Ok(slice)
}

pub fn aead_encrypt(
    alg: &'static ring::aead::Algorithm,
    in_out: &mut Vec<u8>,
    key: &[u8],
    iv: &[u8],
) -> anyhow::Result<()> {
    let key = ring::aead::UnboundKey::new(alg, key)
        .map_err(|_| anyhow::anyhow!("Aead unbound key init error"))?;
    let nonce = ring::aead::Nonce::try_assume_unique_for_key(iv)
        .map_err(|_| anyhow::anyhow!("Aead iv init error"))?;
    let aad = make_teaclave_aad();

    let enc_key = ring::aead::LessSafeKey::new(key);
    enc_key
        .seal_in_place_append_tag(nonce, aad, in_out)
        .map_err(|_| anyhow::anyhow!("Aead seal_in_place_append_tag error"))?;
    Ok(())
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(test_aead_enc_then_dec, test_crypto_info,)
    }

    fn test_aead_enc_then_dec() {
        let plain_text: [u8; 5] = [0xde, 0xff, 0xab, 0xcd, 0x90];
        let key = [0x90u8; AES_GCM_128_KEY_LENGTH];
        let iv = [0x89u8; 12];

        let mut buf = plain_text.to_vec();
        aead_encrypt(&ring::aead::AES_128_GCM, &mut buf, &key, &iv).unwrap();
        let result = aead_decrypt(&ring::aead::AES_128_GCM, &mut buf, &key, &iv).unwrap();
        assert_eq!(&result[..], &plain_text[..]);
    }

    fn test_crypto_info() {
        let key = [0x90u8; AES_GCM_128_KEY_LENGTH];
        let iv = [0x89u8; AES_GCM_128_IV_LENGTH];
        let crypto_info = AesGcm128CryptoInfo { key, iv };

        let plain_text: [u8; 5] = [0xde, 0xff, 0xab, 0xcd, 0x90];
        let mut buf = plain_text.to_vec();

        crypto_info.encrypt(&mut buf).unwrap();
        assert_ne!(&buf[..], &plain_text[..]);

        crypto_info.decrypt(&mut buf).unwrap();
        assert_eq!(&buf[..], &plain_text[..]);
    }
}
