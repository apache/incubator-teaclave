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

use anyhow;
use serde::{Deserialize, Serialize};
use serde_json;
use std::prelude::v1::*;
use uuid::Uuid;

pub trait Storable: Serialize + for<'de> Deserialize<'de> {
    fn key_prefix() -> &'static str;

    fn uuid(&self) -> Uuid;

    fn key_string(&self) -> String {
        format!("{}-{}", Self::key_prefix(), self.uuid().to_string())
    }

    fn key(&self) -> Vec<u8> {
        self.key_string().into_bytes()
    }

    fn match_prefix(key: &str) -> bool {
        key.starts_with(Self::key_prefix())
    }

    fn to_vec(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self)?;
        Ok(bytes)
    }

    fn from_slice(bytes: &[u8]) -> anyhow::Result<Self> {
        let obj = serde_json::from_slice(bytes)?;
        Ok(obj)
    }

    fn external_id(&self) -> String {
        self.key_string()
    }
}
