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

use crate::error::TeaclaveAuthenticationApiError;
use crate::user_db::{DbClient, DbError};
use crate::user_info::UserInfo;
use anyhow::anyhow;
use std::prelude::v1::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::untrusted::time::SystemTimeEx;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationApi, UserLoginRequest, UserLoginResponse, UserRegisterRequest,
    UserRegisterResponse, UserUpdateRequest, UserUpdateResponse,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{bail, ensure, teaclave_service};
use teaclave_types::{TeaclaveServiceResponseResult, UserRole};

#[teaclave_service(
    teaclave_authentication_service,
    TeaclaveAuthenticationApi,
    TeaclaveAuthenticationError
)]
#[derive(Clone)]
pub(crate) struct TeaclaveAuthenticationApiService {
    db_client: DbClient,
    jwt_secret: Vec<u8>,
}

impl TeaclaveAuthenticationApiService {
    pub(crate) fn new(db_client: DbClient, jwt_secret: Vec<u8>) -> Self {
        Self {
            db_client,
            jwt_secret,
        }
    }
}

impl TeaclaveAuthenticationApi for TeaclaveAuthenticationApiService {
    fn user_register(
        &self,
        request: Request<UserRegisterRequest>,
    ) -> TeaclaveServiceResponseResult<UserRegisterResponse> {
        let id: String = request
            .metadata
            .get("id")
            .ok_or_else(|| anyhow!("Missing credential"))?
            .into();
        let token: String = request
            .metadata
            .get("token")
            .ok_or_else(|| anyhow!("Missing credential"))?
            .into();
        let user: UserInfo = match self.db_client.get_user(&id) {
            Ok(value) => value,
            Err(_) => bail!(TeaclaveAuthenticationApiError::PermissionDenied),
        };

        let requester_role = match user.validate_token(&self.jwt_secret, &token) {
            Ok(claims) => claims.get_role(),
            Err(_) => return Err(TeaclaveAuthenticationApiError::PermissionDenied.into()),
        };

        let request = request.message;
        ensure!(
            !request.id.is_empty(),
            TeaclaveAuthenticationApiError::InvalidUserId
        );
        if self.db_client.get_user(&request.id).is_ok() {
            bail!(TeaclaveAuthenticationApiError::InvalidUserId);
        }
        let role = UserRole::new(&request.role, &request.attribute);
        ensure!(
            role != UserRole::Invalid,
            TeaclaveAuthenticationApiError::InvalidRole
        );

        ensure!(
            authorize_user_register(&requester_role, &request),
            TeaclaveAuthenticationApiError::InvalidRole
        );

        let new_user = UserInfo::new(&request.id, &request.password, role);
        match self.db_client.create_user(&new_user) {
            Ok(_) => Ok(UserRegisterResponse {}),
            Err(DbError::UserExist) => Err(TeaclaveAuthenticationApiError::InvalidUserId.into()),
            Err(_) => Err(TeaclaveAuthenticationApiError::ServiceUnavailable.into()),
        }
    }

    fn user_update(
        &self,
        request: Request<UserUpdateRequest>,
    ) -> TeaclaveServiceResponseResult<UserUpdateResponse> {
        let id: String = request
            .metadata
            .get("id")
            .ok_or_else(|| anyhow!("Missing credential"))?
            .into();
        let token: String = request
            .metadata
            .get("token")
            .ok_or_else(|| anyhow!("Missing credential"))?
            .into();
        let user: UserInfo = match self.db_client.get_user(&id) {
            Ok(value) => value,
            Err(_) => bail!(TeaclaveAuthenticationApiError::PermissionDenied),
        };

        let requester_role = match user.validate_token(&self.jwt_secret, &token) {
            Ok(claims) => claims.get_role(),
            Err(_) => return Err(TeaclaveAuthenticationApiError::PermissionDenied.into()),
        };

        let request = request.message;
        ensure!(
            !request.id.is_empty(),
            TeaclaveAuthenticationApiError::InvalidUserId
        );
        if !self.db_client.get_user(&request.id).is_ok() {
            bail!(TeaclaveAuthenticationApiError::InvalidUserId);
        }
        let role = UserRole::new(&request.role, &request.attribute);
        ensure!(
            role != UserRole::Invalid,
            TeaclaveAuthenticationApiError::InvalidRole
        );

        ensure!(
            authorize_user_update(&requester_role, &request),
            TeaclaveAuthenticationApiError::InvalidRole
        );

        let updated_user = UserInfo::new(&request.id, &request.password, role);
        match self.db_client.update_user(&updated_user) {
            Ok(_) => Ok(UserUpdateResponse {}),
            Err(DbError::UserNotExist) => Err(TeaclaveAuthenticationApiError::InvalidUserId.into()),
            Err(_) => Err(TeaclaveAuthenticationApiError::ServiceUnavailable.into()),
        }
    }

    fn user_login(
        &self,
        request: Request<UserLoginRequest>,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        let request = request.message;
        ensure!(
            !request.id.is_empty(),
            TeaclaveAuthenticationApiError::InvalidUserId
        );
        ensure!(
            !request.password.is_empty(),
            TeaclaveAuthenticationApiError::InvalidPassword
        );
        let user = self
            .db_client
            .get_user(&request.id)
            .map_err(|_| TeaclaveAuthenticationApiError::PermissionDenied)?;
        if !user.verify_password(&request.password) {
            bail!(TeaclaveAuthenticationApiError::PermissionDenied)
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| TeaclaveAuthenticationApiError::ServiceUnavailable)?;
            let exp = (now + Duration::from_secs(24 * 60)).as_secs();
            match user.get_token(exp, &self.jwt_secret) {
                Ok(token) => Ok(UserLoginResponse { token }),
                Err(_) => Err(TeaclaveAuthenticationApiError::ServiceUnavailable.into()),
            }
        }
    }
}

fn authorize_user_register(role: &UserRole, request: &UserRegisterRequest) -> bool {
    match role {
        UserRole::PlatformAdmin => true,
        UserRole::DataOwnerManager(s) => {
            let request_role = UserRole::new(&request.role, &request.attribute);
            request_role == UserRole::DataOwner(s.to_owned())
        }
        _ => false,
    }
}

fn authorize_user_update(role: &UserRole, request: &UserUpdateRequest) -> bool {
    match role {
        UserRole::PlatformAdmin => true,
        UserRole::DataOwnerManager(s) => {
            let request_role = UserRole::new(&request.role, &request.attribute);
            request_role == UserRole::DataOwner(s.to_owned())
        }
        _ => false,
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use crate::user_db::*;
    use crate::user_info::*;
    use rand::RngCore;
    use std::collections::HashMap;
    use std::vec;
    use teaclave_rpc::IntoRequest;

    fn get_mock_service() -> TeaclaveAuthenticationApiService {
        let database = Database::open().unwrap();
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        let client = database.get_client();
        crate::create_platform_admin_user(client, "admin", "teaclave").unwrap();

        TeaclaveAuthenticationApiService {
            db_client: database.get_client(),
            jwt_secret,
        }
    }

    pub fn test_user_register() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("id".to_owned(), "admin".to_owned());
        metadata.insert("token".to_owned(), response.token);
        let mut request =
            UserRegisterRequest::new("test_register_id", "test_password", "PlatformAdmin", "")
                .into_request();
        request.metadata = metadata;
        assert!(service.user_register(request).is_ok());
    }

    pub fn test_user_update() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("id".to_owned(), "admin".to_owned());
        metadata.insert("token".to_owned(), response.token);
        let mut request =
            UserRegisterRequest::new("test_update_id", "test_password", "PlatformAdmin", "")
                .into_request();
        request.metadata = metadata.clone();
        assert!(service.user_register(request).is_ok());

        let mut request =
            UserUpdateRequest::new("test_update_id", "updated_password", "PlatformAdmin", "")
                .into_request();
        request.metadata = metadata.clone();
        service.user_update(request).unwrap();

        let mut request =
            UserUpdateRequest::new("test_nonexist_id", "updated_password", "PlatformAdmin", "")
                .into_request();
        request.metadata = metadata;
        assert!(service.user_update(request).is_err());

        let request = UserLoginRequest::new("test_update_id", "updated_password").into_request();
        let response = service.user_login(request);
        assert!(response.is_ok());
    }

    pub fn test_user_login() {
        let service = get_mock_service();

        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("id".to_owned(), "admin".to_owned());
        metadata.insert("token".to_owned(), response.token);

        let mut request =
            UserRegisterRequest::new("test_login_id", "test_password", "FunctionOwner", "")
                .into_request();
        request.metadata = metadata;
        assert!(service.user_register(request).is_ok());

        let request = UserLoginRequest::new("test_login_id", "test_password").into_request();
        let response = service.user_login(request);
        assert!(response.is_ok());

        let token = response.unwrap().token;
        let user = service.db_client.get_user("test_login_id").unwrap();
        assert!(user.validate_token(&service.jwt_secret, &token).is_ok());

        debug!("saved user_info: {:?}", user);
        let request = UserLoginRequest::new("test_login_id", "test_password1").into_request();
        assert!(service.user_login(request).is_err());
    }
}
