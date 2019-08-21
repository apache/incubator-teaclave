// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use mesatee_core::{Error, ErrorKind, Result};
use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey};
use ring::digest;
use std::fmt::Write;

struct FileNonceSequence(Option<aead::Nonce>);

impl FileNonceSequence {
    /// Constructs the sequence allowing `advance()` to be called
    /// `allowed_invocations` times.
    fn new(nonce: aead::Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl aead::NonceSequence for FileNonceSequence {
    fn advance(&mut self) -> core::result::Result<aead::Nonce, ring::error::Unspecified> {
        self.0.take().ok_or(ring::error::Unspecified)
    }
}

pub fn decrypt_data(
    mut data: Vec<u8>,
    aes_key: &[u8],
    aes_nonce: &[u8],
    aes_ad: &[u8],
) -> Result<Vec<u8>> {
    let aead_alg = &aead::AES_256_GCM;
    let ub = UnboundKey::new(aead_alg, aes_key).map_err(|_| Error::from(ErrorKind::CryptoError))?;
    let nonce = Nonce::try_assume_unique_for_key(aes_nonce)
        .map_err(|_| Error::from(ErrorKind::CryptoError))?;
    let filesequence = FileNonceSequence::new(nonce);
    let mut o_key = aead::OpeningKey::new(ub, filesequence);
    let ad = Aad::from(aes_ad);
    let result = o_key.open_in_place(ad, &mut data[..]);
    let decrypted_buffer = result.map_err(|_| Error::from(ErrorKind::CryptoError))?;
    Ok(decrypted_buffer.to_vec())
}

pub fn encrypt_data(
    mut data: Vec<u8>,
    aes_key: &[u8],
    aes_nonce: &[u8],
    aes_ad: &[u8],
) -> Result<Vec<u8>> {
    let aead_alg = &aead::AES_256_GCM;

    if (aes_key.len() != 32) || (aes_nonce.len() != 12) || (aes_ad.len() != 5) {
        return Err(Error::from(ErrorKind::CryptoError));
    }

    let ub = UnboundKey::new(aead_alg, aes_key).map_err(|_| Error::from(ErrorKind::CryptoError))?;
    let nonce = Nonce::try_assume_unique_for_key(aes_nonce)
        .map_err(|_| Error::from(ErrorKind::CryptoError))?;
    let filesequence = FileNonceSequence::new(nonce);
    let mut s_key = aead::SealingKey::new(ub, filesequence);
    let ad = Aad::from(aes_ad);

    let s_result = s_key.seal_in_place_append_tag(ad, &mut data);

    let _ = s_result.map_err(|_| Error::from(ErrorKind::CryptoError))?;

    Ok(data)
}

pub fn cal_hash(data: &[u8]) -> Result<String> {
    let digest_alg = &digest::SHA256;
    let mut ctx = digest::Context::new(digest_alg);
    ctx.update(data);
    let digest_result = ctx.finish();
    let digest_bytes: &[u8] = digest_result.as_ref();
    let mut digest_hex = String::new();
    for &byte in digest_bytes {
        write!(&mut digest_hex, "{:02x}", byte).map_err(|_| Error::from(ErrorKind::Unknown))?;
    }
    Ok(digest_hex)
}
