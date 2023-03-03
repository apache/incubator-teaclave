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

#![feature(strict_provenance)]

mod task_logger;
use task_logger::TaskLogger;

use std::sync::RwLock;

use log::{LevelFilter, Log, Metadata, Record};

struct TeaclaveLogger<T> {
    task_logger: RwLock<Option<TaskLogger>>,
    secondary_logger: Option<T>,
}

impl<T: Log> TeaclaveLogger<T> {
    pub fn log_task(&self, task_logger: TaskLogger) {
        let mut lock = self.task_logger.write().unwrap();
        assert!(lock.is_none(), "only one task is allowed to be logged");

        if let Some(sl) = &self.secondary_logger {
            sl.flush()
        }

        *lock = Some(task_logger);
    }

    pub fn end_logging_task(&self) {
        let mut lock = self.task_logger.write().unwrap();
        assert!(lock.is_some(), "task should be logged before being ended");

        *lock = None;
    }
}

impl<T: Log> Log for TeaclaveLogger<T> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Some(tl) = &*self.task_logger.read().unwrap() {
            tl.enabled(metadata)
        } else if let Some(sl) = &self.secondary_logger {
            sl.enabled(metadata)
        } else {
            false
        }
    }

    fn log(&self, record: &Record) {
        let kv = record.key_values();

        if let Some(v) = kv.get("buffer".into()) {
            let addr = v.to_u64().unwrap();

            if addr == 0 {
                self.end_logging_task();
            } else {
                let task_logger = TaskLogger::new(addr);
                self.log_task(task_logger);
            }

            // Ignore the message when task_id is set
            return;
        }

        let mut lock = self.task_logger.write().unwrap();
        if let Some(ref mut tl) = *lock {
            tl.log(record);
            return;
        }

        if let Some(sl) = &self.secondary_logger {
            sl.log(record)
        }
    }

    fn flush(&self) {
        let mut lock = self.task_logger.write().unwrap();

        if let Some(ref mut tl) = *lock {
            tl.flush();
            return;
        }

        if let Some(sl) = &self.secondary_logger {
            sl.flush()
        }
    }
}

pub struct Builder<T> {
    secondary_logger: Option<T>,
}

impl<T: Log + 'static> Builder<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn secondary_logger(mut self, logger: T) -> Self {
        self.secondary_logger = Some(logger);
        self
    }

    fn build(self) -> TeaclaveLogger<T> {
        let task_logger = RwLock::new(None);

        TeaclaveLogger {
            task_logger,
            secondary_logger: self.secondary_logger,
        }
    }

    pub fn init(self) {
        // Two loggers may be used, so we set the level to trace, and filter inside function log.
        log::set_max_level(LevelFilter::Trace);

        let logger = self.build();

        log::set_boxed_logger(Box::new(logger)).unwrap();
    }
}

impl<T> Default for Builder<T> {
    fn default() -> Self {
        Self {
            secondary_logger: None,
        }
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(test_log,)
    }

    // The logs are not sent to the storage service as the service client is not configured in
    // teaclave_unit_tests.
    fn test_log() {
        log::trace!(task_id = "dummy"; "you should not see this line");
        log::info!("you should see this line from the secondary logger");
        log::warn!(task_id = ""; "");
    }
}
