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

use serde_derive::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AeadConfig {
    #[serde(with = "base64_coder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_coder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_coder")]
    pub ad: Vec<u8>,
}

mod base64_coder {
    // Insert std prelude in the top for the sgx feature
    #[cfg(feature = "mesalock_sgx")]
    use std::prelude::v1::*;

    extern crate base64;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        base64::decode(s).map_err(de::Error::custom)
    }
}
