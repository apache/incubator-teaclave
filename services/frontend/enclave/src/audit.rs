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

use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use std::sync::Arc;

use teaclave_proto::teaclave_management_service::{SaveLogsRequest, TeaclaveManagementClient};
use teaclave_rpc::transport::Channel;
use teaclave_types::Entry;

/// Agent to send audit information to the auditor in the management service.
/// To reduce the network activity, buffer and then send the information every 30 seconds.
pub struct AuditAgent {
    management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
    buffer: Arc<Mutex<Vec<Entry>>>,
}

impl AuditAgent {
    pub fn new(
        management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
        buffer: Arc<Mutex<Vec<Entry>>>,
    ) -> Self {
        Self {
            management_client,
            buffer,
        }
    }

    pub async fn run(&self) {
        loop {
            let mut mutex = self.buffer.lock().await;
            let logs: Vec<Entry> = mutex.drain(..).collect();
            drop(mutex);

            if !logs.is_empty() {
                let request = SaveLogsRequest::new(logs);

                let mut client = self.management_client.lock().await;
                let _ = client.save_logs(request).await;
            }

            sleep(Duration::from_secs(30)).await;
        }
    }
}
