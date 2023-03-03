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

use std::sync::{Arc, Mutex};

use log::{Metadata, Record};

pub struct TaskLogger {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl TaskLogger {
    pub fn new(addr: u64) -> Self {
        let buffer = unsafe { Arc::from_raw(std::ptr::from_exposed_addr_mut(addr as usize)) };

        Self { buffer }
    }
}

impl TaskLogger {
    pub fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    pub fn log(&mut self, record: &Record) {
        let metadata = record.metadata();

        if !self.enabled(metadata) {
            return;
        }

        let mut output = format!("[{}", metadata.level());
        if let Some(module_path) = record.module_path() {
            output += " ";
            output += module_path;
        }
        let output = format!("{}] {}", output, record.args());
        self.buffer.lock().unwrap().push(output);
    }

    pub fn flush(&mut self) {}
}

impl Drop for TaskLogger {
    fn drop(&mut self) {
        self.flush()
    }
}
