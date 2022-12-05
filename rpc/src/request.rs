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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request<T> {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub message: T,
}

impl<T> Request<T> {
    pub fn new(message: T) -> Self {
        Request {
            metadata: HashMap::<String, String>::default(),
            message,
        }
    }

    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        let message = f(self.message);

        Request {
            metadata: self.metadata,
            message,
        }
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.metadata
    }
}

pub trait IntoRequest<T> {
    fn into_request(self) -> Request<T>;
}

impl<T> IntoRequest<T> for T {
    fn into_request(self) -> Request<Self> {
        Request::new(self)
    }
}
