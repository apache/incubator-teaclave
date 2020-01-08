use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend::{
    self, TeaclaveFrontend, UserLoginRequest, UserLoginResponse,
};
use teaclave_service_enclave_utils::teaclave_service;

use teaclave_types::TeaclaveServiceError;

use thiserror::Error;

type Result<T> = std::result::Result<T, TeaclaveServiceError>;

#[derive(Error, Debug)]
pub enum TeaclaveFrontendError {
    #[error("permission denied")]
    PermissionDenied,
}

impl From<TeaclaveFrontendError> for TeaclaveServiceError {
    fn from(error: TeaclaveFrontendError) -> Self {
        TeaclaveServiceError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_frontend, TeaclaveFrontend, TeaclaveFrontendError)]
#[derive(Copy, Clone)]
pub(crate) struct TeaclaveFrontendService;

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn user_login(request: UserLoginRequest) -> Result<UserLoginResponse> {
        if request.id != "test_id" && request.password != "test_password" {
            return Err(TeaclaveFrontendError::PermissionDenied.into());
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
        assert!(TeaclaveFrontendService::user_login(request).is_ok());
    }
}
