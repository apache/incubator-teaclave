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

use crate::channel::SgxTrustedTlsChannel;
use crate::config::SgxTrustedTlsClientConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct Endpoint {
    url: String,
    config: SgxTrustedTlsClientConfig,
}

impl Endpoint {
    pub fn new(url: &str) -> Self {
        let config = SgxTrustedTlsClientConfig::new();
        Self {
            url: url.to_string(),
            config,
        }
    }

    pub fn connect<U, V>(&self) -> Result<SgxTrustedTlsChannel<U, V>>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        SgxTrustedTlsChannel::<U, V>::new(&self.url, &self.config)
    }

    pub fn config(self, config: SgxTrustedTlsClientConfig) -> Self {
        Self {
            url: self.url,
            config,
        }
    }
}
