use std::prelude::v1::*;
use teaclave_proto::teaclave_authentication_service::{
    self, TeaclaveAuthentication, UserLoginRequest, UserLoginResponse,
};
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TeaclaveAuthenticationError {
    #[error("permission denied")]
    PermissionDenied,
}

impl From<TeaclaveAuthenticationError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveAuthenticationError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(
    teaclave_authentication_service,
    TeaclaveAuthentication,
    TeaclaveAuthenticationError
)]
#[derive(Copy, Clone)]
pub(crate) struct TeaclaveAuthenticationService;

impl TeaclaveAuthentication for TeaclaveAuthenticationService {
    fn user_login(_request: UserLoginRequest) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        #[cfg(test_mode)]
        return test_mode::mock_user_login(_request);

        Err(TeaclaveAuthenticationError::PermissionDenied.into())
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
    pub fn mock_user_login(
        request: UserLoginRequest,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        if request.id == "test_id" && request.password == "test_password" {
            let response = UserLoginResponse {
                token: "test_token".to_string(),
            };
            return Ok(response);
        }
        Err(TeaclaveAuthenticationError::PermissionDenied.into())
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
        assert!(TeaclaveAuthenticationService::user_login(request).is_ok());
    }
}
