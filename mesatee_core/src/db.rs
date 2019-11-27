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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use cfg_if::cfg_if;

// Use target specific definitions here
cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        use std::sync::SgxRwLock as RwLock;
    } else {
        use std::sync::RwLock;
    }
}

use crate::Result;
use std::collections::HashMap;
use std::hash;

pub struct Memdb<K: Clone + Eq + hash::Hash, V: Clone> {
    hashmap: RwLock<HashMap<K, V>>,
}

impl<K: Clone + Eq + hash::Hash, V: Clone> Memdb<K, V> {
    pub fn open() -> Result<Self> {
        Ok(Self {
            hashmap: RwLock::new(HashMap::<K, V>::new()),
        })
    }

    pub fn set(&self, key: &K, value: &V) -> Result<Option<V>> {
        let mut hashmap = self.hashmap.write()?;
        Ok(hashmap.insert(key.to_owned(), value.to_owned()))
    }

    pub fn get(&self, key: &K) -> Result<Option<V>> {
        let hashmap = self.hashmap.read()?;
        Ok(hashmap.get(key).cloned())
    }

    pub fn del(&self, key: &K) -> Result<Option<V>> {
        let mut hashmap = self.hashmap.write()?;
        Ok(hashmap.remove(key))
    }
}
