use std::prelude::v1::*;
use teaclave_proto::teaclave_authorization_service::{
    self, TeaclaveAuthorization, UserLoginRequest, UserLoginResponse,
};
use teaclave_service_enclave_utils::teaclave_service;

use teaclave_types::TeaclaveServiceError;

use thiserror::Error;

type Result<T> = std::result::Result<T, TeaclaveServiceError>;

#[derive(Error, Debug)]
pub enum TeaclaveAuthorizationError {
    #[error("permission denied")]
    PermissionDenied,
}

impl From<TeaclaveAuthorizationError> for TeaclaveServiceError {
    fn from(error: TeaclaveAuthorizationError) -> Self {
        TeaclaveServiceError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_authorization_service, TeaclaveAuthorization, TeaclaveAuthorizationError)]
#[derive(Copy, Clone)]
pub(crate) struct TeaclaveAuthorizationService;

impl TeaclaveAuthorization for TeaclaveAuthorizationService {
    fn user_login(request: UserLoginRequest) -> Result<UserLoginResponse> {
        if request.id != "test_id" && request.password != "test_password" {
            return Err(TeaclaveAuthorizationError::PermissionDenied.into());
        }
        let response = UserLoginResponse {
            token: "test_token".to_string(),
        };
        Ok(response)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn test_user_login() {
        let request = UserLoginRequest {
            id: "test_id".to_string(),
            password: "test_password".to_string(),
        };
        assert!(TeaclaveAuthorizationService::user_login(request).is_ok());
    }
}
