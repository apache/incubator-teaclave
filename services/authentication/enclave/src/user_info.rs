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

use anyhow::Result;
use jsonwebtoken::{self, Header, Validation};
use rand::prelude::RngCore;
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::prelude::v1::*;
use std::vec;

const SALT_LEN: usize = 16;
const PASS_LEN: usize = digest::SHA512_OUTPUT_LEN;
static PBKDF2_ALG: ring::pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;
const ITER_LEN: u32 = 100_000;
pub const ISSUER_NAME: &str = "TeaClave";
pub static JWT_ALG: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;
pub const JWT_SECRET_LEN: usize = 512;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub id: String,
    pub salt: Vec<u8>,                 // size is SALT_LEN
    pub salted_password_hash: Vec<u8>, // size is PASS_LEN
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // user id
    pub sub: String,
    // issuer: ISSUER_NAME
    pub iss: String,
    // expiration time
    // (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 + some seconds)
    pub exp: i64,
}

impl UserInfo {
    pub fn new_register_user(id: &str, password: &str) -> Option<UserInfo> {
        let mut rng = rand::thread_rng();
        let mut salt: [u8; SALT_LEN] = [0; SALT_LEN];
        rng.fill_bytes(&mut salt);
        let mut salted_password_hash = [0; PASS_LEN];
        let iter = match NonZeroU32::new(ITER_LEN) {
            Some(value) => value,
            None => return None,
        };
        pbkdf2::derive(
            PBKDF2_ALG,
            iter,
            &salt,
            password.as_bytes(),
            &mut salted_password_hash,
        );
        Some(Self {
            id: id.to_string(),
            salt: salt.to_vec(),
            salted_password_hash: salted_password_hash.to_vec(),
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let mut salt: [u8; SALT_LEN] = [0; SALT_LEN];
        let mut salted_password_hash: [u8; PASS_LEN] = [0; PASS_LEN];
        if self.salt.len() != SALT_LEN || self.salted_password_hash.len() != PASS_LEN {
            return false;
        }
        let iter = match NonZeroU32::new(ITER_LEN) {
            Some(value) => value,
            None => return false,
        };
        salt.copy_from_slice(&self.salt[0..SALT_LEN]);
        salted_password_hash.copy_from_slice(&self.salted_password_hash[0..PASS_LEN]);
        pbkdf2::verify(
            PBKDF2_ALG,
            iter,
            &salt,
            password.as_bytes(),
            &salted_password_hash,
        )
        .is_ok()
    }

    pub fn get_token(&self, exp: i64, secret: &[u8]) -> Result<String> {
        let my_claims = Claims {
            sub: self.id.clone(),
            iss: ISSUER_NAME.to_string(),
            exp,
        };
        let mut header = Header::default();
        header.alg = JWT_ALG;
        let token = jsonwebtoken::encode(&header, &my_claims, secret)?;
        Ok(token)
    }

    pub fn validate_token(&self, secret: &[u8], token: &str) -> bool {
        let validation = Validation {
            iss: Some(ISSUER_NAME.to_string()),
            sub: Some(self.id.to_string()),
            algorithms: vec![JWT_ALG],
            ..Default::default()
        };
        let token_data = jsonwebtoken::decode::<Claims>(token, secret, &validation);
        token_data.is_ok()
    }
}
