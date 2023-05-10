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

use teaclave_types::UserRole;
use tonic::{
    codegen::InterceptedService, service::Interceptor, transport::Channel, IntoRequest, Request,
    Status,
};

pub type CredentialService = InterceptedService<Channel, UserCredential>;

// To verify authentication credentials of the request.
#[derive(Debug, Default, Clone)]
pub struct UserCredential {
    pub id: String,
    pub token: String,
    pub role: UserRole,
}

impl Interceptor for UserCredential {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let mut req = request.into_request();
        let meta = req.metadata_mut();
        meta.insert("id", self.id.parse().unwrap());
        meta.insert("token", self.token.parse().unwrap());
        meta.insert("role", self.role.to_string().parse().unwrap());
        Ok(req)
    }
}

impl UserCredential {
    pub fn new(id: impl ToString, token: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            token: token.to_string(),
            role: UserRole::default(),
        }
    }

    pub fn with_role(id: impl ToString, token: impl ToString, role: UserRole) -> Self {
        Self {
            id: id.to_string(),
            token: token.to_string(),
            role,
        }
    }
}
