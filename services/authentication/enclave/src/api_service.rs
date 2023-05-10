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

use crate::error::AuthenticationError;
use crate::error::AuthenticationServiceError;
use crate::user_db::DbClient;
use crate::user_info::UserInfo;

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use std::untrusted::time::SystemTimeEx;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_rpc::{Request, Response};
use teaclave_service_enclave_utils::{bail, ensure};
use teaclave_types::{TeaclaveServiceResponseResult, UserRole};
#[derive(Clone)]
pub(crate) struct TeaclaveAuthenticationApiService {
    db_client: Arc<Mutex<DbClient>>,
    jwt_secret: Vec<u8>,
}

impl TeaclaveAuthenticationApiService {
    pub(crate) fn new(db_client: DbClient, jwt_secret: Vec<u8>) -> Self {
        Self {
            db_client: Arc::new(Mutex::new(db_client)),
            jwt_secret,
        }
    }

    fn validate_user_credential(
        &self,
        id: &str,
        token: &str,
    ) -> Result<UserRole, AuthenticationError> {
        let user: UserInfo = match self.db_client.lock().unwrap().get_user(id) {
            Ok(value) => value,
            Err(_) => bail!(AuthenticationError::InvalidUserId),
        };

        if token.is_empty() {
            bail!(AuthenticationError::InvalidToken);
        }

        match user.validate_token(&self.jwt_secret, token) {
            Ok(claims) => Ok(claims.get_role()),
            Err(_) => bail!(AuthenticationError::IncorrectToken),
        }
    }

    fn validate_credential_in_request<T>(
        &self,
        request: &Request<T>,
    ) -> Result<UserRole, AuthenticationServiceError> {
        let id: String = request
            .metadata()
            .get("id")
            .and_then(|x| x.to_str().ok())
            .ok_or(AuthenticationServiceError::MissingUserId)?
            .into();
        let token: String = request
            .metadata()
            .get("token")
            .and_then(|x| x.to_str().ok())
            .ok_or(AuthenticationServiceError::MissingToken)?
            .into();
        let requester_role = self.validate_user_credential(&id, &token)?;
        Ok(requester_role)
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveAuthenticationApi for TeaclaveAuthenticationApiService {
    async fn user_register(
        &self,
        request: Request<UserRegisterRequest>,
    ) -> TeaclaveServiceResponseResult<UserRegisterResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let request = request.get_ref();
        ensure!(
            !request.id.is_empty(),
            AuthenticationServiceError::InvalidUserId
        );
        if self.db_client.lock().unwrap().get_user(&request.id).is_ok() {
            bail!(AuthenticationServiceError::UserIdExist);
        }
        let role = UserRole::new(&request.role, &request.attribute);
        ensure!(
            role != UserRole::Invalid,
            AuthenticationServiceError::InvalidRole
        );

        ensure!(
            authorize_user_register(&requester_role, request),
            AuthenticationServiceError::PermissionDenied
        );

        let new_user = UserInfo::new(&request.id, &request.password, role);
        match self.db_client.lock().unwrap().create_user(&new_user) {
            Ok(_) => Ok(Response::new(UserRegisterResponse {})),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
        }
    }

    async fn user_update(
        &self,
        request: Request<UserUpdateRequest>,
    ) -> TeaclaveServiceResponseResult<UserUpdateResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let request = request.get_ref();
        ensure!(
            !request.id.is_empty(),
            AuthenticationServiceError::InvalidUserId
        );
        if self
            .db_client
            .lock()
            .unwrap()
            .get_user(&request.id)
            .is_err()
        {
            bail!(AuthenticationServiceError::InvalidUserId);
        }
        let role = UserRole::new(&request.role, &request.attribute);
        ensure!(
            role != UserRole::Invalid,
            AuthenticationServiceError::InvalidRole
        );

        ensure!(
            authorize_user_update(&requester_role, request),
            AuthenticationServiceError::PermissionDenied
        );

        let updated_user = UserInfo::new(&request.id, &request.password, role);
        match self.db_client.lock().unwrap().update_user(&updated_user) {
            Ok(_) => Ok(Response::new(UserUpdateResponse {})),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
        }
    }

    async fn user_login(
        &self,
        request: Request<UserLoginRequest>,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        let request = request.get_ref();
        ensure!(!request.id.is_empty(), AuthenticationError::InvalidUserId);
        ensure!(
            !request.password.is_empty(),
            AuthenticationError::InvalidPassword
        );
        let user = self
            .db_client
            .lock()
            .unwrap()
            .get_user(&request.id)
            .map_err(|_| AuthenticationError::UserIdNotFound)?;
        if !user.verify_password(&request.password) {
            bail!(AuthenticationError::IncorrectPassword)
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| AuthenticationServiceError::Service(e.into()))?;
            let exp = (now + Duration::from_secs(24 * 60 * 60)).as_secs();
            match user.get_token(exp, &self.jwt_secret) {
                Ok(token) => Ok(Response::new(UserLoginResponse { token })),
                Err(e) => bail!(AuthenticationServiceError::Service(e)),
            }
        }
    }

    async fn user_change_password(
        &self,
        request: Request<UserChangePasswordRequest>,
    ) -> TeaclaveServiceResponseResult<UserChangePasswordResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let id: String = request
            .metadata()
            .get("id")
            .and_then(|x| x.to_str().ok())
            .unwrap()
            .into();
        let request = request.get_ref();
        ensure!(
            !request.password.is_empty(),
            AuthenticationError::InvalidPassword
        );
        let updated_user = UserInfo::new(&id, &request.password, requester_role);

        match self.db_client.lock().unwrap().update_user(&updated_user) {
            Ok(_) => Ok(Response::new(UserChangePasswordResponse {})),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
        }
    }

    async fn reset_user_password(
        &self,
        request: Request<ResetUserPasswordRequest>,
    ) -> TeaclaveServiceResponseResult<ResetUserPasswordResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let request = request.get_ref();
        ensure!(
            !request.id.is_empty(),
            AuthenticationServiceError::InvalidUserId
        );
        let user = self
            .db_client
            .lock()
            .unwrap()
            .get_user(&request.id)
            .map_err(|_| AuthenticationServiceError::PermissionDenied)?;

        ensure!(
            authorize_reset_user_password(&requester_role, &user),
            AuthenticationServiceError::PermissionDenied
        );

        let mut encode_buffer = uuid::Uuid::encode_buffer();
        let new_password = uuid::Uuid::new_v4()
            .to_simple()
            .encode_lower(&mut encode_buffer);
        let updated_user = UserInfo::new(&request.id, new_password, user.role);
        match self.db_client.lock().unwrap().update_user(&updated_user) {
            Ok(_) => Ok(Response::new(ResetUserPasswordResponse {
                password: new_password.to_string(),
            })),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> TeaclaveServiceResponseResult<DeleteUserResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let request = request.get_ref();
        ensure!(
            !request.id.is_empty(),
            AuthenticationServiceError::InvalidUserId
        );
        let user = self
            .db_client
            .lock()
            .unwrap()
            .get_user(&request.id)
            .map_err(|_| AuthenticationServiceError::PermissionDenied)?;

        ensure!(
            authorize_delete_user(&requester_role, &user),
            AuthenticationServiceError::PermissionDenied
        );
        match self.db_client.lock().unwrap().delete_user(&request.id) {
            Ok(_) => Ok(Response::new(DeleteUserResponse {})),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
        }
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> TeaclaveServiceResponseResult<ListUsersResponse> {
        let requester_role = self.validate_credential_in_request(&request)?;

        let request = request.get_ref();
        ensure!(
            !request.id.is_empty(),
            AuthenticationServiceError::InvalidUserId
        );

        ensure!(
            authorize_list_users(&requester_role, request),
            AuthenticationServiceError::PermissionDenied
        );

        let users = match requester_role {
            UserRole::PlatformAdmin => self.db_client.lock().unwrap().list_users(),
            _ => self
                .db_client
                .lock()
                .unwrap()
                .list_users_by_attribute(&request.id),
        };

        match users {
            Ok(ids) => Ok(Response::new(ListUsersResponse { ids })),
            Err(e) => bail!(AuthenticationServiceError::Service(e.into())),
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
        UserRole::FunctionOwner => {
            let request_role = UserRole::new(&request.role, &request.attribute);
            matches!(request_role, UserRole::DataOwnerManager(_))
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

fn authorize_reset_user_password(role: &UserRole, target_user: &UserInfo) -> bool {
    match role {
        UserRole::PlatformAdmin => true,
        UserRole::DataOwnerManager(s) => {
            let request_role = &target_user.role;
            *request_role == UserRole::DataOwner(s.to_owned())
        }
        _ => false,
    }
}

fn authorize_delete_user(role: &UserRole, target_user: &UserInfo) -> bool {
    match role {
        UserRole::PlatformAdmin => true,
        UserRole::DataOwnerManager(s) => {
            let request_role = &target_user.role;
            *request_role == UserRole::DataOwner(s.to_owned())
        }
        _ => false,
    }
}

fn authorize_list_users(role: &UserRole, request: &ListUsersRequest) -> bool {
    match role {
        UserRole::PlatformAdmin => true,
        UserRole::DataOwnerManager(s) => s == &request.id,
        UserRole::FunctionOwner => false,
        _ => false,
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use crate::user_db::*;
    use crate::user_info::*;
    use rand::RngCore;
    use std::vec;
    use teaclave_rpc::{IntoRequest, MetadataMap};

    fn get_mock_service() -> TeaclaveAuthenticationApiService {
        let database = Database::open("").unwrap();
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        let client = database.get_client();
        crate::create_platform_admin_user(client, "admin", "teaclave").unwrap();

        TeaclaveAuthenticationApiService {
            db_client: Arc::new(Mutex::new(database.get_client())),
            jwt_secret,
        }
    }

    pub async fn test_user_register() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());
        let mut request =
            UserRegisterRequest::new("test_register_id", "test_password", "PlatformAdmin", "")
                .into_request();
        let meta = request.metadata_mut();
        *meta = metadata;
        assert!(service.user_register(request).await.is_ok());
    }

    pub async fn test_user_update() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());
        let mut request =
            UserRegisterRequest::new("test_update_id", "test_password", "PlatformAdmin", "")
                .into_request();
        let meta = request.metadata_mut();
        *meta = metadata.clone();
        assert!(service.user_register(request).await.is_ok());

        let mut request =
            UserUpdateRequest::new("test_update_id", "updated_password", "PlatformAdmin", "")
                .into_request();
        let meta = request.metadata_mut();
        *meta = metadata.clone();
        service.user_update(request).await.unwrap();

        let mut request =
            UserUpdateRequest::new("test_nonexist_id", "updated_password", "PlatformAdmin", "")
                .into_request();
        let meta = request.metadata_mut();
        *meta = metadata;
        assert!(service.user_update(request).await.is_err());

        let request = UserLoginRequest::new("test_update_id", "updated_password").into_request();
        let response = service.user_login(request).await;
        assert!(response.is_ok());
    }

    pub async fn test_user_login() {
        let service = get_mock_service();

        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());

        let mut request =
            UserRegisterRequest::new("test_login_id", "test_password", "FunctionOwner", "")
                .into_request();
        let meta = request.metadata_mut();
        *meta = metadata;
        assert!(service.user_register(request).await.is_ok());

        let request = UserLoginRequest::new("test_login_id", "test_password").into_request();
        let response = service.user_login(request).await;
        assert!(response.is_ok());

        let token = response.unwrap().into_inner().token;
        let user = service
            .db_client
            .lock()
            .unwrap()
            .get_user("test_login_id")
            .unwrap();
        assert!(user.validate_token(&service.jwt_secret, &token).is_ok());

        debug!("saved user_info: {:?}", user);
        let request = UserLoginRequest::new("test_login_id", "test_password1").into_request();
        assert!(service.user_login(request).await.is_err());
    }

    pub async fn test_user_change_password() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());
        let mut request = UserRegisterRequest::new(
            "test_user_change_password_id",
            "test_password",
            "PlatformAdmin",
            "",
        )
        .into_request();
        *request.metadata_mut() = metadata;
        assert!(service.user_register(request).await.is_ok());

        let request =
            UserLoginRequest::new("test_user_change_password_id", "test_password").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();
        let mut metadata = MetadataMap::new();
        metadata.insert("id", "test_user_change_password_id".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());

        let mut request = UserChangePasswordRequest::new("updated_password").into_request();
        *request.metadata_mut() = metadata.clone();
        service.user_change_password(request).await.unwrap();

        let mut request = UserChangePasswordRequest::new("").into_request();
        *request.metadata_mut() = metadata;

        assert!(service.user_change_password(request).await.is_err());

        let request = UserLoginRequest::new("test_user_change_password_id", "updated_password")
            .into_request();
        let response = service.user_login(request).await;
        assert!(response.is_ok());
    }

    pub async fn test_reset_user_password() {
        let service = get_mock_service();
        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());
        let mut request = UserRegisterRequest::new(
            "test_reset_user_password_id",
            "test_password",
            "PlatformAdmin",
            "",
        )
        .into_request();
        *request.metadata_mut() = metadata.clone();
        assert!(service.user_register(request).await.is_ok());

        let mut request =
            ResetUserPasswordRequest::new("test_reset_user_password_id").into_request();
        *request.metadata_mut() = metadata.clone();
        let response = service.reset_user_password(request).await;
        assert!(response.is_ok());

        let request = UserLoginRequest::new(
            "test_reset_user_password_id",
            response.unwrap().into_inner().password,
        )
        .into_request();
        let response = service.user_login(request).await;
        assert!(response.is_ok());
    }

    pub async fn test_delete_user() {
        let service = get_mock_service();

        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();

        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());

        let mut request =
            UserRegisterRequest::new("test_delete_user_id", "test_password", "FunctionOwner", "")
                .into_request();
        *request.metadata_mut() = metadata;
        assert!(service.user_register(request).await.is_ok());

        let request = UserLoginRequest::new("test_delete_user_id", "test_password").into_request();
        let response = service.user_login(request).await;
        assert!(response.is_ok());

        let token = response.unwrap().into_inner().token;
        let user = service
            .db_client
            .lock()
            .unwrap()
            .get_user("test_delete_user_id")
            .unwrap();
        assert!(user.validate_token(&service.jwt_secret, &token).is_ok());

        let request = UserLoginRequest::new("admin", "teaclave").into_request();
        let response = service.user_login(request).await.unwrap().into_inner();
        let mut metadata = MetadataMap::new();
        metadata.insert("id", "admin".parse().unwrap());
        metadata.insert("token", response.token.parse().unwrap());
        let mut request = DeleteUserRequest::new("test_delete_user_id").into_request();
        *request.metadata_mut() = metadata;
        assert!(service.delete_user(request).await.is_ok());

        debug!("saved user_info: {:?}", user);
        let request = UserLoginRequest::new("test_delete_user_id", "test_password").into_request();
        assert!(service.user_login(request).await.is_err());
    }
}
