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

use crate::teaclave_frontend_service_proto as proto;
use anyhow::{Error, Result};
use core::convert::TryInto;
use std::collections::HashMap;
use teaclave_types::{
    Entry, Executor, ExecutorType, ExternalID, FileAuthTag, FileCrypto, Function, FunctionArgument,
    FunctionArguments, FunctionBuilder, FunctionInput, FunctionOutput, OwnerList, TaskFileOwners,
};
use url::Url;

pub use proto::teaclave_frontend_client::TeaclaveFrontendClient;
pub use proto::teaclave_frontend_server::TeaclaveFrontend;
pub use proto::teaclave_frontend_server::TeaclaveFrontendServer;
pub use proto::*;

impl_custom_server!(TeaclaveFrontendServer, TeaclaveFrontend);
impl_custom_client!(TeaclaveFrontendClient);

impl RegisterInputFileRequest {
    pub fn new(url: Url, cmac: FileAuthTag, crypto: impl Into<FileCrypto>) -> Self {
        Self {
            url: url.as_str().to_string(),
            cmac: cmac.to_bytes(),
            crypto_info: Some(crypto.into().into()),
        }
    }
}

impl UpdateInputFileRequest {
    pub fn new(data_id: ExternalID, url: Url) -> Self {
        Self {
            data_id: data_id.to_string(),
            url: url.as_str().to_string(),
        }
    }
}

impl RegisterInputFileResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl UpdateInputFileResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl RegisterOutputFileRequest {
    pub fn new(url: Url, crypto: impl Into<FileCrypto>) -> Self {
        Self {
            url: url.as_str().to_string(),
            crypto_info: Some(crypto.into().into()),
        }
    }
}

impl UpdateOutputFileRequest {
    pub fn new(data_id: ExternalID, url: Url) -> Self {
        Self {
            data_id: data_id.to_string(),
            url: url.as_str().to_string(),
        }
    }
}

impl RegisterOutputFileResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl UpdateOutputFileResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl RegisterFusionOutputRequest {
    pub fn new(owner_list: impl Into<OwnerList>) -> Self {
        Self {
            owner_list: owner_list.into().into(),
        }
    }
}

impl RegisterFusionOutputResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl RegisterInputFromOutputRequest {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl RegisterInputFromOutputResponse {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl GetInputFileRequest {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl GetInputFileResponse {
    pub fn new(owner: OwnerList, cmac: FileAuthTag) -> Self {
        Self {
            owner: owner.into(),
            cmac: cmac.to_bytes(),
        }
    }
}

impl GetOutputFileRequest {
    pub fn new(data_id: ExternalID) -> Self {
        Self {
            data_id: data_id.to_string(),
        }
    }
}

impl GetOutputFileResponse {
    pub fn new(owner: OwnerList, cmac: Option<FileAuthTag>) -> Self {
        Self {
            owner: owner.into(),
            cmac: cmac.map_or_else(Vec::new, |cmac| cmac.to_bytes()),
        }
    }
}

#[derive(Default)]
pub struct RegisterFunctionRequestBuilder {
    request: RegisterFunctionRequest,
}

impl RegisterFunctionRequestBuilder {
    pub fn new() -> Self {
        let request = RegisterFunctionRequest {
            executor_type: ExecutorType::Builtin.to_string(),
            public: true,
            usage_quota: -1,
            ..Default::default()
        };

        Self { request }
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.request.name = name.to_string();
        self
    }

    pub fn description(mut self, description: impl ToString) -> Self {
        self.request.description = description.to_string();
        self
    }

    pub fn executor_type(mut self, executor_type: ExecutorType) -> Self {
        self.request.executor_type = executor_type.to_string();
        self
    }

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.request.payload = payload;
        self
    }

    pub fn public(mut self, public: bool) -> Self {
        self.request.public = public;
        self
    }

    pub fn arguments(mut self, args: Vec<FunctionArgument>) -> Self {
        self.request.arguments = args.into_iter().map(|x| x.into()).collect();
        self
    }

    pub fn inputs(mut self, inputs: Vec<FunctionInput>) -> Self {
        self.request.inputs = inputs.into_iter().map(|x| x.into()).collect();
        self
    }

    pub fn outputs(mut self, outputs: Vec<FunctionOutput>) -> Self {
        self.request.outputs = outputs.into_iter().map(|x| x.into()).collect();
        self
    }

    pub fn user_allowlist(mut self, user_allowlist: Vec<String>) -> Self {
        self.request.user_allowlist = user_allowlist;
        self
    }

    pub fn usage_quota(mut self, usage_quota: Option<i32>) -> Self {
        self.request.usage_quota = usage_quota.unwrap_or(-1);
        self
    }

    pub fn build(self) -> RegisterFunctionRequest {
        self.request
    }
}

// We explicitly construct Function here in case of missing any field
impl std::convert::TryFrom<RegisterFunctionRequest> for FunctionBuilder {
    type Error = Error;
    fn try_from(request: RegisterFunctionRequest) -> Result<Self> {
        Ok(FunctionBuilder::new()
            .name(request.name)
            .description(request.description)
            .public(request.public)
            .executor_type(request.executor_type.try_into()?)
            .payload(request.payload)
            .arguments(
                request
                    .arguments
                    .into_iter()
                    .map(FunctionArgument::try_from)
                    .collect::<Result<_>>()?,
            )
            .inputs(
                request
                    .inputs
                    .into_iter()
                    .map(FunctionInput::try_from)
                    .collect::<Result<_>>()?,
            )
            .outputs(
                request
                    .outputs
                    .into_iter()
                    .map(FunctionOutput::try_from)
                    .collect::<Result<_>>()?,
            )
            .user_allowlist(request.user_allowlist)
            .usage_quota((request.usage_quota >= 0).then_some(request.usage_quota)))
    }
}

impl RegisterFunctionResponse {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

#[derive(Default)]
pub struct UpdateFunctionRequestBuilder {
    request: UpdateFunctionRequest,
}

impl UpdateFunctionRequestBuilder {
    pub fn new() -> Self {
        let request = UpdateFunctionRequest {
            executor_type: ExecutorType::Builtin.to_string(),
            public: true,
            usage_quota: -1,
            ..Default::default()
        };

        Self { request }
    }

    pub fn function_id(mut self, id: ExternalID) -> Self {
        self.request.function_id = id.to_string();
        self
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.request.name = name.to_string();
        self
    }

    pub fn description(mut self, description: impl ToString) -> Self {
        self.request.description = description.to_string();
        self
    }

    pub fn executor_type(mut self, executor_type: ExecutorType) -> Self {
        self.request.executor_type = executor_type.to_string();
        self
    }

    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.request.payload = payload;
        self
    }

    pub fn public(mut self, public: bool) -> Self {
        self.request.public = public;
        self
    }

    pub fn arguments(mut self, args: Vec<FunctionArgument>) -> Self {
        self.request.arguments = args
            .into_iter()
            .map(proto::FunctionArgument::from)
            .collect();
        self
    }

    pub fn inputs(mut self, inputs: Vec<FunctionInput>) -> Self {
        self.request.inputs = inputs.into_iter().map(proto::FunctionInput::from).collect();
        self
    }

    pub fn outputs(mut self, outputs: Vec<FunctionOutput>) -> Self {
        self.request.outputs = outputs
            .into_iter()
            .map(proto::FunctionOutput::from)
            .collect();
        self
    }

    pub fn user_allowlist(mut self, user_allowlist: Vec<String>) -> Self {
        self.request.user_allowlist = user_allowlist;
        self
    }

    pub fn usage_quota(mut self, usage_quota: Option<i32>) -> Self {
        self.request.usage_quota = usage_quota.unwrap_or(-1);
        self
    }

    pub fn build(self) -> UpdateFunctionRequest {
        self.request
    }
}

// We explicitly construct Function here in case of missing any field
impl std::convert::TryFrom<UpdateFunctionRequest> for FunctionBuilder {
    type Error = Error;
    fn try_from(request: UpdateFunctionRequest) -> Result<Self> {
        let function_id: ExternalID = request.function_id.try_into()?;
        Ok(FunctionBuilder::new()
            .id(function_id.uuid)
            .name(request.name)
            .description(request.description)
            .public(request.public)
            .executor_type(request.executor_type.try_into()?)
            .payload(request.payload)
            .arguments(
                request
                    .arguments
                    .into_iter()
                    .map(FunctionArgument::try_from)
                    .collect::<Result<_>>()?,
            )
            .inputs(
                request
                    .inputs
                    .into_iter()
                    .map(FunctionInput::try_from)
                    .collect::<Result<_>>()?,
            )
            .outputs(
                request
                    .outputs
                    .into_iter()
                    .map(FunctionOutput::try_from)
                    .collect::<Result<_>>()?,
            )
            .user_allowlist(request.user_allowlist)
            .usage_quota((request.usage_quota >= 0).then_some(request.usage_quota)))
    }
}

impl UpdateFunctionResponse {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

impl GetFunctionRequest {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

impl GetFunctionUsageStatsRequest {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

impl DeleteFunctionRequest {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

impl DisableFunctionRequest {
    pub fn new(function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
        }
    }
}

impl CreateTaskRequest {
    pub fn new() -> Self {
        Self {
            executor: Executor::default().to_string(),
            function_arguments: FunctionArguments::default().into_string(),
            ..Default::default()
        }
    }

    pub fn function_id(self, function_id: ExternalID) -> Self {
        Self {
            function_id: function_id.to_string(),
            ..self
        }
    }

    pub fn function_arguments(self, function_arguments: impl Into<FunctionArguments>) -> Self {
        Self {
            function_arguments: function_arguments.into().into_string(),
            ..self
        }
    }

    pub fn executor(self, executor: Executor) -> Self {
        Self {
            executor: executor.to_string(),
            ..self
        }
    }

    pub fn inputs_ownership(self, map: impl Into<TaskFileOwners>) -> Self {
        Self {
            inputs_ownership: to_proto_ownership(map.into()),
            ..self
        }
    }

    pub fn outputs_ownership(self, map: impl Into<TaskFileOwners>) -> Self {
        Self {
            outputs_ownership: to_proto_ownership(map.into()),
            ..self
        }
    }
}

impl CreateTaskResponse {
    pub fn new(task_id: ExternalID) -> Self {
        Self {
            task_id: task_id.to_string(),
        }
    }
}

impl GetTaskRequest {
    pub fn new(task_id: ExternalID) -> Self {
        Self {
            task_id: task_id.to_string(),
        }
    }
}

impl AssignDataRequest {
    pub fn new(
        task_id: ExternalID,
        inputs: HashMap<String, ExternalID>,
        outputs: HashMap<String, ExternalID>,
    ) -> Self {
        let inputs = to_proto_file_ids(inputs);
        let outputs = to_proto_file_ids(outputs);
        Self {
            task_id: task_id.to_string(),
            inputs,
            outputs,
        }
    }
}

impl ApproveTaskRequest {
    pub fn new(task_id: ExternalID) -> Self {
        Self {
            task_id: task_id.to_string(),
        }
    }
}

impl InvokeTaskRequest {
    pub fn new(task_id: ExternalID) -> Self {
        Self {
            task_id: task_id.to_string(),
        }
    }
}

impl CancelTaskRequest {
    pub fn new(task_id: ExternalID) -> Self {
        Self {
            task_id: task_id.to_string(),
        }
    }
}

impl std::convert::TryFrom<proto::FunctionInput> for FunctionInput {
    type Error = Error;

    fn try_from(proto: proto::FunctionInput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
            optional: proto.optional,
        };

        Ok(ret)
    }
}

impl From<FunctionInput> for proto::FunctionInput {
    fn from(input: FunctionInput) -> Self {
        Self {
            name: input.name,
            description: input.description,
            optional: input.optional,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionOutput> for FunctionOutput {
    type Error = Error;

    fn try_from(proto: proto::FunctionOutput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
            optional: proto.optional,
        };

        Ok(ret)
    }
}

impl From<FunctionOutput> for proto::FunctionOutput {
    fn from(output: FunctionOutput) -> Self {
        Self {
            name: output.name,
            description: output.description,
            optional: output.optional,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionArgument> for FunctionArgument {
    type Error = Error;

    fn try_from(proto: proto::FunctionArgument) -> Result<Self> {
        let ret = Self {
            key: proto.key,
            default_value: proto.default_value,
            allow_overwrite: proto.allow_overwrite,
        };

        Ok(ret)
    }
}

impl From<FunctionArgument> for proto::FunctionArgument {
    fn from(arg: FunctionArgument) -> Self {
        Self {
            key: arg.key,
            default_value: arg.default_value,
            allow_overwrite: arg.allow_overwrite,
        }
    }
}

impl From<Function> for GetFunctionResponse {
    fn from(function: Function) -> Self {
        Self {
            name: function.name,
            description: function.description,
            owner: function.owner.to_string(),
            executor_type: function.executor_type.to_string(),
            payload: function.payload,
            public: function.public,
            arguments: function.arguments.into_iter().map(|x| x.into()).collect(),
            inputs: function.inputs.into_iter().map(|x| x.into()).collect(),
            outputs: function.outputs.into_iter().map(|x| x.into()).collect(),
            user_allowlist: function.user_allowlist,
        }
    }
}

pub fn from_proto_ownership(proto: Vec<proto::OwnerList>) -> TaskFileOwners {
    proto
        .into_iter()
        .map(|ol| (ol.data_name, ol.uids))
        .collect()
}

pub fn to_proto_ownership(ownership: TaskFileOwners) -> Vec<proto::OwnerList> {
    ownership
        .into_iter()
        .map(|(name, ol)| proto::OwnerList {
            data_name: name,
            uids: ol.into(),
        })
        .collect()
}

pub fn to_proto_file_ids(map: HashMap<String, ExternalID>) -> Vec<proto::DataMap> {
    map.into_iter()
        .map(|(name, ext_id)| proto::DataMap {
            data_name: name,
            data_id: ext_id.to_string(),
        })
        .collect()
}

pub fn from_proto_file_ids(vector: Vec<proto::DataMap>) -> Result<HashMap<String, ExternalID>> {
    vector
        .into_iter()
        .map(|item| {
            item.data_id
                .clone()
                .try_into()
                .map(|ext_id| (item.data_name, ext_id))
        })
        .collect()
}

impl QueryAuditLogsRequest {
    pub fn new(query: String, limit: usize) -> Self {
        Self {
            query,
            limit: limit as u64,
        }
    }
}

impl QueryAuditLogsResponse {
    pub fn new(entries: Vec<Entry>) -> Self {
        let logs: Vec<crate::teaclave_common_proto::Entry> = entries
            .into_iter()
            .map(crate::teaclave_common_proto::Entry::from)
            .collect();

        Self { logs }
    }
}
