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

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
extern crate sgx_tstd as std;

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{anyhow, ensure, Context, Result};
use protected_fs::ProtectedFile;
use rand::prelude::RngCore;
use ring::aead;
use serde::{Deserialize, Serialize};
use std::format;
use std::path::Path;

const AES_GCM_128_KEY_LENGTH: usize = 16;
const AES_GCM_128_IV_LENGTH: usize = 12;

const AES_GCM_256_KEY_LENGTH: usize = 32;
const AES_GCM_256_IV_LENGTH: usize = 12;
const TEACLAVE_FILE_128_ROOT_KEY_LENGTH: usize = 16;
const CMAC_LENGTH: usize = 16;
type CMac = [u8; CMAC_LENGTH];


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AesGcm256Key {
    pub key: [u8; AES_GCM_256_KEY_LENGTH],
    pub iv: [u8; AES_GCM_256_IV_LENGTH],
}

impl AesGcm256Key {
    pub const SCHEMA: &'static str = "aes-gcm-256";

    pub fn new(in_key: &[u8], in_iv: &[u8]) -> Result<Self> {
        ensure!(
            in_key.len() == AES_GCM_256_KEY_LENGTH,
            "Invalid key length for AesGcm256: {}",
            in_key.len()
        );
        ensure!(
            in_iv.len() == AES_GCM_256_IV_LENGTH,
            "Invalid iv length for AesGcm256: {}",
            in_iv.len()
        );
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        key.copy_from_slice(in_key);
        iv.copy_from_slice(in_iv);

        Ok(AesGcm256Key { key, iv })
    }

    pub fn from_hex(in_key: impl AsRef<str>, in_iv: impl AsRef<str>) -> Result<Self> {
        let key = hex::decode(in_key.as_ref()).context("Illegal AesGcm256 key provided")?;
        let iv = hex::decode(in_iv.as_ref()).context("Illegal AesGcm256 iv provided")?;
        Self::new(&key, &iv)
    }

    pub fn random() -> Self {
        Self::default()
    }

    pub fn decrypt(&self, in_out: &mut Vec<u8>) -> Result<[u8; CMAC_LENGTH]> {
        let plaintext_len = aead_decrypt(&aead::AES_256_GCM, in_out, &self.key, &self.iv)?.len();
        let mut cmac:CMac = [0u8; CMAC_LENGTH];
        cmac.copy_from_slice(&in_out[plaintext_len..]);
        in_out.truncate(plaintext_len);
        Ok(cmac)
    }

    pub fn encrypt(&self, in_out: &mut Vec<u8>) -> Result<[u8; CMAC_LENGTH]> {
        aead_encrypt(&aead::AES_256_GCM, in_out, &self.key, &self.iv)?;
        let mut cmac:CMac = [0u8; CMAC_LENGTH];
        let n = in_out.len();
        let cybertext_len = n - CMAC_LENGTH;
        cmac.copy_from_slice(&in_out[cybertext_len..]);
        Ok(cmac)
    }
}

impl Default for AesGcm256Key {
    fn default() -> Self {
        let mut key = [0u8; AES_GCM_256_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_256_IV_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);

        Self { key, iv }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AesGcm128Key {
    pub key: [u8; AES_GCM_128_KEY_LENGTH],
    pub iv: [u8; AES_GCM_128_IV_LENGTH],
}

impl AesGcm128Key {
    pub const SCHEMA: &'static str = "aes-gcm-128";

    pub fn new(in_key: &[u8], in_iv: &[u8]) -> Result<Self> {
        ensure!(
            in_key.len() == AES_GCM_128_KEY_LENGTH,
            "Invalid key length for AesGcm128: {}",
            in_key.len()
        );

        ensure!(
            in_iv.len() == AES_GCM_128_IV_LENGTH,
            "Invalid iv length for AesGcm128: {}",
            in_iv.len()
        );

        let mut key = [0u8; AES_GCM_128_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_128_IV_LENGTH];
        key.copy_from_slice(in_key);
        iv.copy_from_slice(in_iv);

        Ok(AesGcm128Key { key, iv })
    }

    pub fn from_hex(in_key: impl AsRef<str>, in_iv: impl AsRef<str>) -> Result<Self> {
        let key = hex::decode(in_key.as_ref()).context("Illegal AesGcm128 key provided")?;
        let iv = hex::decode(in_iv.as_ref()).context("Illegal AesGcm128 iv provided")?;
        Self::new(&key, &iv)
    }

    pub fn random() -> Self {
        Self::default()
    }

    pub fn decrypt(&self, in_out: &mut Vec<u8>) -> Result<[u8; CMAC_LENGTH]> {
        let plaintext_len = aead_decrypt(&aead::AES_128_GCM, in_out, &self.key, &self.iv)?.len();
        let mut cmac:CMac = [0u8; CMAC_LENGTH];
        cmac.copy_from_slice(&in_out[plaintext_len..]);
        in_out.truncate(plaintext_len);
        Ok(cmac)
    }

    pub fn encrypt(&self, in_out: &mut Vec<u8>) -> Result<[u8; CMAC_LENGTH]> {
        aead_encrypt(&aead::AES_128_GCM, in_out, &self.key, &self.iv)?;
        let mut cmac:CMac = [0u8; CMAC_LENGTH];
        let n = in_out.len();
        let cybertext_len = n - CMAC_LENGTH;
        cmac.copy_from_slice(&in_out[cybertext_len..]);
        Ok(cmac)   
    }
}

impl Default for AesGcm128Key {
    fn default() -> Self {
        let mut key = [0u8; AES_GCM_128_KEY_LENGTH];
        let mut iv = [0u8; AES_GCM_128_IV_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut iv);

        Self { key, iv }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TeaclaveFile128Key {
    pub key: [u8; TEACLAVE_FILE_128_ROOT_KEY_LENGTH],
}

impl TeaclaveFile128Key {
    pub const SCHEMA: &'static str = "teaclave-file-128";

    pub fn random() -> Self {
        Self::default()
    }

    pub fn new(in_key: &[u8]) -> Result<Self> {
        ensure!(
            in_key.len() == TEACLAVE_FILE_128_ROOT_KEY_LENGTH,
            "Invalid key length for teaclave_file_128: {}",
            in_key.len()
        );
        let mut key = [0u8; TEACLAVE_FILE_128_ROOT_KEY_LENGTH];
        key.copy_from_slice(in_key);

        Ok(TeaclaveFile128Key { key })
    }

    pub fn decrypt<P: AsRef<Path>>(&self, path: P, out: &mut Vec<u8>) -> Result<[u8; CMAC_LENGTH]> {
        use std::io::Read;
        let mut file = ProtectedFile::open_ex(path.as_ref(), &self.key)?;
        let cmac = file.current_meta_gmac()?;
        file.read_to_end(out)?;
        Ok(cmac)
    }

    pub fn encrypt<P: AsRef<Path>>(&self, path: P, content: &[u8]) -> Result<[u8; CMAC_LENGTH]> {
        use std::io::Write;
        let mut file = ProtectedFile::create_ex(path.as_ref(), &self.key)?;
        let cmac = file.current_meta_gmac()?;
        file.write_all(content)?;
        Ok(cmac)
    }
}

impl Default for TeaclaveFile128Key {
    fn default() -> Self {
        let mut key = [0u8; TEACLAVE_FILE_128_ROOT_KEY_LENGTH];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);

        TeaclaveFile128Key { key }
    }
}

pub fn aead_decrypt<'a>(
    alg: &'static aead::Algorithm,
    in_out: &'a mut [u8],
    key: &[u8],
    iv: &[u8],
) -> Result<&'a mut [u8]> {
    let key =
        aead::UnboundKey::new(alg, key).map_err(|_| anyhow!("Aead unbound key init error"))?;
    let nonce =
        aead::Nonce::try_assume_unique_for_key(iv).map_err(|_| anyhow!("Aead iv init error"))?;
    let aad = aead::Aad::from([0u8; 8]);

    let dec_key = aead::LessSafeKey::new(key);
    let slice = dec_key
        .open_in_place(nonce, aad, in_out)
        .map_err(|_| anyhow!("Aead open_in_place error"))?;
    Ok(slice)
}

pub fn aead_encrypt(
    alg: &'static aead::Algorithm,
    in_out: &mut Vec<u8>,
    key: &[u8],
    iv: &[u8],
) -> Result<()> {
    let key =
        aead::UnboundKey::new(alg, key).map_err(|_| anyhow!("Aead unbound key init error"))?;
    let nonce =
        aead::Nonce::try_assume_unique_for_key(iv).map_err(|_| anyhow!("Aead iv init error"))?;
    let aad = aead::Aad::from([0u8; 8]);

    let enc_key = aead::LessSafeKey::new(key);
    enc_key
        .seal_in_place_append_tag(nonce, aad, in_out)
        .map_err(|_| anyhow!("Aead seal_in_place_append_tag error"))?;
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
        aead_encrypt(&aead::AES_128_GCM, &mut buf, &key, &iv).unwrap();
        let result = aead_decrypt(&aead::AES_128_GCM, &mut buf, &key, &iv).unwrap();
        assert_eq!(&result[..], &plain_text[..]);
    }

    fn test_crypto_info() {
        let key = [0x90u8; AES_GCM_128_KEY_LENGTH];
        let iv = [0x89u8; AES_GCM_128_IV_LENGTH];
        let crypto_info = AesGcm128Key { key, iv };

        let plain_text: [u8; 5] = [0xde, 0xff, 0xab, 0xcd, 0x90];
        let mut buf = plain_text.to_vec();

        crypto_info.encrypt(&mut buf).unwrap();
        assert_ne!(&buf[..], &plain_text[..]);

        crypto_info.decrypt(&mut buf).unwrap();
        assert_eq!(&buf[..], &plain_text[..]);
    }
}
