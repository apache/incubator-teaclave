use crate::user_db::{DbClient, DbError};
use crate::user_info::UserInfo;
use std::prelude::v1::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::untrusted::time::SystemTimeEx;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationApi, UserLoginRequest, UserLoginResponse, UserRegisterRequest,
    UserRegisterResponse,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclaveAuthenticationApiError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid userid")]
    InvalidUserId,
    #[error("invalid password")]
    InvalidPassword,
    #[error("service unavailable")]
    ServiceUnavailable,
}

impl From<TeaclaveAuthenticationApiError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveAuthenticationApiError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

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
        let request = request.message;
        if request.id.is_empty() {
            return Err(TeaclaveAuthenticationApiError::InvalidUserId.into());
        }
        if self.db_client.get_user(&request.id).is_ok() {
            return Err(TeaclaveAuthenticationApiError::InvalidUserId.into());
        }
        let new_user = UserInfo::new(&request.id, &request.password);
        match self.db_client.create_user(&new_user) {
            Ok(_) => Ok(UserRegisterResponse {}),
            Err(DbError::UserExist) => Err(TeaclaveAuthenticationApiError::InvalidUserId.into()),
            Err(_) => Err(TeaclaveAuthenticationApiError::ServiceUnavailable.into()),
        }
    }

    fn user_login(
        &self,
        request: Request<UserLoginRequest>,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        let request = request.message;
        if request.id.is_empty() {
            return Err(TeaclaveAuthenticationApiError::InvalidUserId.into());
        }
        if request.password.is_empty() {
            return Err(TeaclaveAuthenticationApiError::InvalidPassword.into());
        }
        let user = self
            .db_client
            .get_user(&request.id)
            .map_err(|_| TeaclaveAuthenticationApiError::PermissionDenied)?;
        if !user.verify_password(&request.password) {
            Err(TeaclaveAuthenticationApiError::PermissionDenied.into())
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

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use crate::user_db::*;
    use crate::user_info::*;
    use rand::RngCore;
    use std::vec;

    fn get_mock_service() -> TeaclaveAuthenticationApiService {
        let database = Database::open().unwrap();
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        TeaclaveAuthenticationApiService {
            db_client: database.get_client(),
            jwt_secret,
        }
    }

    pub fn test_user_register() {
        let request = UserRegisterRequest::new("test_register_id", "test_password");
        let request = Request::new(request);
        let service = get_mock_service();
        assert!(service.user_register(request).is_ok());
    }

    pub fn test_user_login() {
        let service = get_mock_service();
        let request = UserRegisterRequest::new("test_login_id", "test_password");
        let request = Request::new(request);
        assert!(service.user_register(request).is_ok());
        let request = UserLoginRequest::new("test_login_id", "test_password");
        let request = Request::new(request);
        let response = service.user_login(request);
        assert!(response.is_ok());
        let token = response.unwrap().token;
        let user = service.db_client.get_user("test_login_id").unwrap();
        assert!(user.validate_token(&service.jwt_secret, &token));

        info!("saved user_info: {:?}", user);
        let request = UserLoginRequest::new("test_login_id", "test_password1");
        let request = Request::new(request);
        assert!(service.user_login(request).is_err());
    }
}
