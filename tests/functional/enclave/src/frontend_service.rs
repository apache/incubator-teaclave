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
use futures::FutureExt;
use std::convert::TryFrom;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_common::{ExecutorCommand, ExecutorStatus};
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::CredentialService;
use teaclave_test_utils::async_test_case;
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

async fn authorized_client() -> TeaclaveFrontendClient<CredentialService> {
    let mut api_client = create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR)
        .await
        .unwrap();
    let cred = login(&mut api_client, USERNAME, TEST_PASSWORD)
        .await
        .unwrap();
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred)
        .await
        .unwrap()
}

async fn unauthorized_client() -> TeaclaveFrontendClient<CredentialService> {
    let cred = UserCredential::new(USERNAME, "InvalidToken");
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred)
        .await
        .unwrap()
}

#[async_test_case]
async fn test_register_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url.clone(), cmac, crypto_info);
    let mut client = authorized_client().await;
    let response = client.register_input_file(request).await;
    assert!(response.is_ok());

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let mut client = unauthorized_client().await;
    let response = client.register_input_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_update_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let mut client = authorized_client().await;
    let response = client.register_input_file(request).await;
    assert!(response.is_ok());

    let old_data_id = response.unwrap().into_inner().data_id;
    let new_url = Url::parse("https://external-storage.com/filepath-new?presigned_token").unwrap();
    let update_request =
        UpdateInputFileRequest::new(old_data_id.clone().try_into().unwrap(), new_url);
    let mut client = authorized_client().await;
    let update_response = client.update_input_file(update_request).await;
    assert!(update_response.is_ok());
    assert!(old_data_id != update_response.unwrap().into_inner().data_id);
}

#[async_test_case]
async fn test_register_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url.clone(), crypto_info);
    let mut client = authorized_client().await;
    let response = client.register_output_file(request).await;
    assert!(response.is_ok());

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let mut client = unauthorized_client().await;
    let response = client.register_output_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_update_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let mut client = authorized_client().await;
    let response = client.register_output_file(request).await;
    assert!(response.is_ok());

    let old_data_id = response.unwrap().into_inner().data_id;
    let new_url = Url::parse("https://external-storage.com/filepath-new?presigned_token").unwrap();
    let update_request =
        UpdateOutputFileRequest::new(old_data_id.clone().try_into().unwrap(), new_url);
    let mut client = authorized_client().await;
    let update_response = client.update_output_file(update_request).await;
    assert!(update_response.is_ok());
    assert!(old_data_id != update_response.unwrap().into_inner().data_id);
}

#[async_test_case]
async fn test_register_fusion_output() {
    let request = RegisterFusionOutputRequest::new(vec!["frontend_user", "mock_user"]);
    let mut client = authorized_client().await;
    let response = client.register_fusion_output(request).await;
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["frontend_user", "mock_user"]);
    let mut client = unauthorized_client().await;
    let response = client.register_fusion_output(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_register_input_from_output() {
    let output_id = ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    let request = RegisterInputFromOutputRequest::new(output_id.clone());
    let mut client = authorized_client().await;
    let response = client.register_input_from_output(request).await;
    assert!(response.is_ok());

    let request = RegisterInputFromOutputRequest::new(output_id);
    let mut client = unauthorized_client().await;
    let response = client.register_input_from_output(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_get_output_file() {
    let mut client = authorized_client().await;

    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let data_id: ExternalID = response.data_id.try_into().unwrap();

    let request = GetOutputFileRequest::new(data_id.clone());
    client.get_output_file(request).await.unwrap();

    let request = GetOutputFileRequest::new(data_id);
    let mut client = unauthorized_client().await;
    let response = client.get_output_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_get_input_file() {
    let mut client = authorized_client().await;

    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    let data_id: ExternalID = response.data_id.try_into().unwrap();

    let request = GetInputFileRequest::new(data_id.clone());
    client.get_input_file(request).await.unwrap();

    let request = GetInputFileRequest::new(data_id);
    let mut client = unauthorized_client().await;
    let response = client.get_input_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_register_function() {
    let request = RegisterFunctionRequestBuilder::new().build();
    let mut client = authorized_client().await;
    let response = client.register_function(request).await;
    assert!(response.is_ok());

    let request = RegisterFunctionRequestBuilder::new().build();
    let mut client = unauthorized_client().await;
    let response = client.register_function(request).await;
    assert!(response.is_err());

    // Wait for the logs to be sent to the auditor
    std::thread::sleep(std::time::Duration::from_secs(35));

    let function_name = "register_function";

    // query by user name
    let request = QueryAuditLogsRequest::new("user:".to_string() + USERNAME, 100);
    let response = authorized_client()
        .await
        .query_audit_logs(request)
        .await
        .unwrap();

    let logs: Vec<_> = response
        .into_inner()
        .logs
        .into_iter()
        .map(|e| Entry::try_from(e).unwrap())
        .filter(|e| e.message() == function_name)
        .collect();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].result());

    // query by function name stored in the message
    let request = QueryAuditLogsRequest::new("message:".to_string() + function_name, 100);
    let response = authorized_client()
        .await
        .query_audit_logs(request)
        .await
        .unwrap();
    let logs: Vec<_> = response
        .into_inner()
        .logs
        .into_iter()
        .map(|e| Entry::try_from(e).unwrap())
        .collect();
    assert_eq!(logs.len(), 2);
    assert!(logs[0].user().contains(USERNAME));
    assert!(logs[0].result());

    assert!(!logs[1].result());

    let request = QueryAuditLogsRequest::new("message:".to_string() + "authenticate", 100);
    let response = authorized_client()
        .await
        .query_audit_logs(request)
        .await
        .unwrap();
    let logs: Vec<_> = response
        .into_inner()
        .logs
        .into_iter()
        .map(|e| Entry::try_from(e).unwrap())
        .collect();
    // "authenticate" message will only show in the entry with false result and empty user
    for log in logs {
        assert_eq!(log.user(), "");
        assert!(!log.result());
    }
}

#[async_test_case]
async fn test_get_function() {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();

    let request = GetFunctionRequest::new(function_id.clone());
    let mut client = authorized_client().await;
    let response = client.get_function(request).await;
    assert!(response.is_ok());

    let request = GetFunctionRequest::new(function_id);
    let mut client = unauthorized_client().await;
    let response = client.get_function(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_create_task() {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id.clone())
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" =>  vec!["frontend_user", "mock_user"]));
    let mut client = authorized_client().await;
    let response = client.create_task(request).await;
    assert!(response.is_ok());

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    let mut client = unauthorized_client().await;
    let response = client.create_task(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_get_task() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();
    let request = GetTaskRequest::new(task_id.clone());
    let response = client.get_task(request).await;
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    let mut client = unauthorized_client().await;
    let response = client.get_task(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_assign_data() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));

    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_id: ExternalID = response.data_id.try_into().unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => output_id.clone()),
    );
    let mut unauthorized_client = unauthorized_client().await;
    let response = unauthorized_client.assign_data(request).await;
    assert!(response.is_err());

    let request = AssignDataRequest::new(task_id, hashmap!(), hashmap!("output" => output_id));
    let response = client.assign_data(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_approve_task() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_id: ExternalID = response.data_id.try_into().unwrap();

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).await.unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    let mut unauthorized_client = unauthorized_client().await;
    let response = unauthorized_client.approve_task(request).await;
    assert!(response.is_err());

    let request = ApproveTaskRequest::new(task_id);
    let response = client.approve_task(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_invoke_task() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_id: ExternalID = response.data_id.try_into().unwrap();
    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).await.unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).await.unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let mut unauthorized_client = unauthorized_client().await;
    let response = unauthorized_client.invoke_task(request).await;
    assert!(response.is_err());

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).await;
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).await.unwrap().into_inner();
    assert_eq!(response.status, i32_from_task_status(TaskStatus::Staged));

    let mut scheduler_client = get_scheduler_client().await;
    let executor_id = Uuid::new_v4().to_string();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_cancel_task() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_id: ExternalID = response.data_id.try_into().unwrap();

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).await.unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).await.unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).await;
    assert!(response.is_ok());

    let mut scheduler_client = get_scheduler_client().await;

    std::thread::sleep(std::time::Duration::from_secs(5));

    let executor_id = Uuid::new_v4();
    let request = HeartbeatRequest::new(executor_id, ExecutorStatus::Idle);

    let response = scheduler_client
        .heartbeat(request)
        .await
        .unwrap()
        .into_inner();
    assert!(response.command == ExecutorCommand::NewTask as i32);

    let request = CancelTaskRequest::new(task_id.clone());
    let response = client.cancel_task(request).await;
    assert!(response.is_ok());

    std::thread::sleep(std::time::Duration::from_secs(3));

    let pull_task_request = PullTaskRequest {
        executor_id: executor_id.to_string(),
    };
    let response = scheduler_client.pull_task(pull_task_request).await;
    log::debug!("response: {:?}", response);

    assert!(response.is_err());

    std::thread::sleep(std::time::Duration::from_secs(3));

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).await.unwrap().into_inner();
    assert_eq!(response.status, i32_from_task_status(TaskStatus::Canceled));
}

#[async_test_case]
async fn test_fail_task() {
    let mut client = authorized_client().await;
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let external_outfile_url =
        Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let external_outfile_crypto = FileCrypto::default();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "arg1_value"))
        .executor(Executor::MesaPy)
        .outputs_ownership(hashmap!("output" => vec!["frontend_user"]));
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id: ExternalID = response.task_id.try_into().unwrap();

    let request = RegisterOutputFileRequest::new(external_outfile_url, external_outfile_crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_id: ExternalID = response.data_id.try_into().unwrap();

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!("output" => output_id));
    client.assign_data(request).await.unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).await.unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).await;
    assert!(response.is_ok());

    let mut scheduler_client = get_scheduler_client().await;

    std::thread::sleep(std::time::Duration::from_secs(5));

    let executor_id = Uuid::new_v4();
    let request = HeartbeatRequest::new(executor_id, ExecutorStatus::Idle);

    let response = scheduler_client
        .heartbeat(request)
        .await
        .unwrap()
        .into_inner();
    assert!(response.command == ExecutorCommand::NewTask as i32);

    let pull_task_request = PullTaskRequest {
        executor_id: executor_id.to_string(),
    };
    let response = scheduler_client.pull_task(pull_task_request).await.unwrap();
    log::debug!("response: {:?}", response);

    let request = HeartbeatRequest::new(executor_id, ExecutorStatus::Executing);
    let response = scheduler_client
        .heartbeat(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("response: {:?}", response);
    assert!(response.command == ExecutorCommand::NoAction as i32);

    std::thread::sleep(std::time::Duration::from_secs(33));

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).await.unwrap().into_inner();

    assert_eq!(response.status, i32_from_task_status(TaskStatus::Failed));
}
