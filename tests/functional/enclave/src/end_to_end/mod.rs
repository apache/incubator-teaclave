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

use crate::utils::*;
use teaclave_proto::teaclave_common::i32_from_task_status;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_types::*;
use url::Url;
mod builtin_echo;
mod builtin_gbdt_train;
use teaclave_rpc::CredentialService;

async fn get_task(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    task_id: &ExternalID,
) -> GetTaskResponse {
    let request = GetTaskRequest::new(task_id.clone());
    let response = client.get_task(request).await.unwrap().into_inner();
    log::debug!("Get task: {:?}", response);
    response
}

async fn get_task_until(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    task_id: &ExternalID,
    status: TaskStatus,
) -> String {
    let status = i32_from_task_status(status);
    loop {
        let response = get_task(client, task_id).await;
        std::thread::sleep(std::time::Duration::from_secs(1));

        if response.status == status {
            let result = teaclave_types::TaskResult::try_from(response.result).unwrap();
            match result {
                TaskResult::Ok(outputs) => {
                    let ret_val = String::from_utf8(outputs.return_value).unwrap();
                    log::debug!("Task returns: {:?}", ret_val);
                    return ret_val;
                }
                TaskResult::Err(failure) => {
                    log::error!("Task failed, reason: {:?}", failure);
                    return failure.to_string();
                }
                TaskResult::NotReady => unreachable!(),
            }
        }
    }
}

async fn approve_task(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    task_id: &ExternalID,
) -> anyhow::Result<()> {
    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).await?;
    Ok(())
}

async fn invoke_task(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    task_id: &ExternalID,
) -> anyhow::Result<()> {
    let request = InvokeTaskRequest::new(task_id.clone());
    client.invoke_task(request).await?;
    Ok(())
}
