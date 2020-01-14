use std::prelude::v1::*;
use teaclave_proto::teaclave_authentication_service::{
    self, TeaclaveAuthentication, UserAuthorizeRequest, UserAuthorizeResponse, UserLoginRequest,
    UserLoginResponse,
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
#[derive(Clone)]
pub(crate) struct TeaclaveAuthenticationService;

impl TeaclaveAuthentication for TeaclaveAuthenticationService {
    fn user_login(
        &self,
        _request: UserLoginRequest,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        #[cfg(test_mode)]
        return test_mode::mock_user_login(_request);

        Err(TeaclaveAuthenticationError::PermissionDenied.into())
    }

    fn user_authorize(
        &self,
        _request: UserAuthorizeRequest,
    ) -> TeaclaveServiceResponseResult<UserAuthorizeResponse> {
        #[cfg(test_mode)]
        return test_mode::mock_user_authorize(_request);

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

    pub fn mock_user_authorize(
        request: UserAuthorizeRequest,
    ) -> TeaclaveServiceResponseResult<UserAuthorizeResponse> {
        if request.credential.id == "test_id" && request.credential.token == "test_token" {
            let response = UserAuthorizeResponse { accept: true };
            return Ok(response);
        }
        Err(TeaclaveAuthenticationError::PermissionDenied.into())
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_proto::teaclave_common::UserCredential;

    pub fn test_user_login() {
        let request = UserLoginRequest {
            id: "test_id".to_string(),
            password: "test_password".to_string(),
        };
        let service = TeaclaveAuthenticationService;
        assert!(service.user_login(request).is_ok());
    }

    pub fn test_user_authorize() {
        let credential = UserCredential {
            id: "test_id".to_string(),
            token: "test_token".to_string(),
        };

        let request = UserAuthorizeRequest { credential };
        let service = TeaclaveAuthenticationService;
        assert!(service.user_authorize(request).is_ok());
    }
}
