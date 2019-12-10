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

// Credential can be different from the one defined in the proto, as long as the from trait is implementd.
// Follow the same pattern as other protos
pub struct Credential {
    pub user_id: String,
    pub user_token: String,
}

pub mod proto {
    #![allow(warnings)]
    #![allow(clippy)]
    #![allow(unknown_lints)]
    include!("prost_generated/cred_proto.rs");
}

impl Credential {
    pub fn new(user_id: &str, user_token: &str) -> Self {
        Credential {
            user_id: user_id.to_owned(),
            user_token: user_token.to_owned(),
        }
    }
    pub fn auth(&self) -> bool {
        true
    }

    pub fn get_user_id(&self) -> String {
        self.user_id.to_owned()
    }
}

impl From<proto::Credential> for Credential {
    fn from(config: proto::Credential) -> Self {
        Credential {
            user_id: config.user_id,
            user_token: config.user_token,
        }
    }
}

impl From<Credential> for proto::Credential {
    fn from(config: Credential) -> Self {
        proto::Credential {
            user_id: config.user_id,
            user_token: config.user_token,
        }
    }
}

impl proto::Credential {
    pub fn get_creds(&self) -> Credential {
        Credential::from(self.clone())
    }
}
