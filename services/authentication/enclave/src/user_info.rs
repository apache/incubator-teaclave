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
use jsonwebtoken as jwt;
use rand::prelude::RngCore;
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::num;
use std::prelude::v1::*;
use std::vec;

const SALT_LEN: usize = 16;
const PASSWORD_DIGEST_LEN: usize = digest::SHA512_OUTPUT_LEN;
const PBKDF2_ITERATIONS: u32 = 100_000;
static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;

pub(crate) const ISSUER_NAME: &str = "Teaclave";
pub(crate) static JWT_ALG: jwt::Algorithm = jwt::Algorithm::HS512;
pub(crate) const JWT_SECRET_LEN: usize = 512;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct UserInfo {
    pub id: String,
    pub salt: Vec<u8>,
    pub salted_password_hash: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    // user id
    pub sub: String,
    // issuer
    pub iss: String,
    // expiration time
    pub exp: u64,
}

impl UserInfo {
    pub(crate) fn new(id: &str, password: &str) -> Self {
        let mut rng = rand::thread_rng();
        let mut salt = vec![0u8; SALT_LEN];
        rng.fill_bytes(&mut salt);
        let mut salted_password_hash = vec![0u8; PASSWORD_DIGEST_LEN];
        let pbkdf2_iterations = num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap();
        pbkdf2::derive(
            PBKDF2_ALG,
            pbkdf2_iterations,
            &salt,
            password.as_bytes(),
            &mut salted_password_hash,
        );
        Self {
            id: id.to_string(),
            salt,
            salted_password_hash,
        }
    }

    pub(crate) fn verify_password(&self, password: &str) -> bool {
        let pbkdf2_iterations = num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap();
        pbkdf2::verify(
            PBKDF2_ALG,
            pbkdf2_iterations,
            &self.salt,
            password.as_bytes(),
            &self.salted_password_hash,
        )
        .is_ok()
    }

    pub(crate) fn get_token(&self, exp: u64, secret: &[u8]) -> Result<String> {
        let iss = ISSUER_NAME.to_string();
        let claims = Claims {
            sub: self.id.to_string(),
            iss,
            exp,
        };
        let mut header = jwt::Header::default();
        header.alg = JWT_ALG;
        let secret = jwt::EncodingKey::from_secret(secret);
        let token = jwt::encode(&header, &claims, &secret)?;
        Ok(token)
    }

    pub(crate) fn validate_token(&self, secret: &[u8], token: &str) -> bool {
        let iss = ISSUER_NAME.to_string();
        let mut validation = jwt::Validation::new(JWT_ALG);
        validation.iss = Some(iss);
        validation.sub = Some(self.id.to_string());
        let secret = jwt::DecodingKey::from_secret(secret);
        jwt::decode::<Claims>(token, &secret, &validation).is_ok()
    }
}
