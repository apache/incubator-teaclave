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

use crate::acs::init_memory_enforcer;
use crate::error::TeaclavAccessControlError;
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_rpc::{Request, Response};
use teaclave_types::TeaclaveServiceResponseResult;

use std::sync::{Arc, RwLock};

use casbin::{CoreApi, Enforcer};

#[derive(Clone)]
pub(crate) struct TeaclaveAccessControlService {
    api_enforcer: Arc<RwLock<Enforcer>>,
}

impl TeaclaveAccessControlService {
    pub(crate) async fn new() -> Self {
        let api_enforcer = Arc::new(RwLock::new(init_memory_enforcer().await.unwrap()));
        TeaclaveAccessControlService { api_enforcer }
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveAccessControl for TeaclaveAccessControlService {
    async fn authorize_api(
        &self,
        request: Request<AuthorizeApiRequest>,
    ) -> TeaclaveServiceResponseResult<AuthorizeApiResponse> {
        let e = self.api_enforcer.read().unwrap();
        let request = request.into_inner();

        let accept = e
            .enforce((request.user_role, request.api))
            .map_err(|_| TeaclavAccessControlError::AccessControlError)?;

        Ok(Response::new(AuthorizeApiResponse { accept }))
    }
}
