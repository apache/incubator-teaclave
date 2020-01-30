use crate::user_db::{DbClient, DbError};
use crate::user_info::{UserInfo, JWT_SECRET_LEN};
use rand::RngCore;
use std::prelude::v1::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::untrusted::time::SystemTimeEx;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthentication, UserAuthenticateRequest, UserAuthenticateResponse, UserLoginRequest,
    UserLoginResponse, UserRegisterRequest, UserRegisterResponse,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclaveAuthenticationError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("invalid userid")]
    InvalidUserId,
    #[error("invalid password")]
    InvalidPassword,
    #[error("service unavailable")]
    ServiceUnavailable,
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
pub(crate) struct TeaclaveAuthenticationService {
    db_client: DbClient,
    jwt_secret: Vec<u8>,
}

impl TeaclaveAuthenticationService {
    pub(crate) fn new(db_client: DbClient) -> anyhow::Result<Self> {
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        Ok(Self {
            db_client,
            jwt_secret,
        })
    }
}

impl TeaclaveAuthentication for TeaclaveAuthenticationService {
    fn user_register(
        &self,
        request: Request<UserRegisterRequest>,
    ) -> TeaclaveServiceResponseResult<UserRegisterResponse> {
        let request = request.message;
        if request.id.is_empty() {
            return Err(TeaclaveAuthenticationError::InvalidUserId.into());
        }
        if self.db_client.get_user(&request.id).is_ok() {
            return Err(TeaclaveAuthenticationError::InvalidUserId.into());
        }
        let new_user = UserInfo::new(&request.id, &request.password);
        match self.db_client.create_user(&new_user) {
            Ok(_) => Ok(UserRegisterResponse {}),
            Err(DbError::UserExist) => Err(TeaclaveAuthenticationError::InvalidUserId.into()),
            Err(_) => Err(TeaclaveAuthenticationError::ServiceUnavailable.into()),
        }
    }

    fn user_login(
        &self,
        request: Request<UserLoginRequest>,
    ) -> TeaclaveServiceResponseResult<UserLoginResponse> {
        let request = request.message;
        if request.id.is_empty() {
            return Err(TeaclaveAuthenticationError::InvalidUserId.into());
        }
        if request.password.is_empty() {
            return Err(TeaclaveAuthenticationError::InvalidPassword.into());
        }
        let user = self
            .db_client
            .get_user(&request.id)
            .map_err(|_| TeaclaveAuthenticationError::PermissionDenied)?;
        if !user.verify_password(&request.password) {
            Err(TeaclaveAuthenticationError::PermissionDenied.into())
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| TeaclaveAuthenticationError::ServiceUnavailable)?;
            let exp = (now + Duration::from_secs(24 * 60)).as_secs();
            match user.get_token(exp, &self.jwt_secret) {
                Ok(token) => Ok(UserLoginResponse { token }),
                Err(_) => Err(TeaclaveAuthenticationError::ServiceUnavailable.into()),
            }
        }
    }

    fn user_authenticate(
        &self,
        request: Request<UserAuthenticateRequest>,
    ) -> TeaclaveServiceResponseResult<UserAuthenticateResponse> {
        let request = request.message;
        if request.credential.id.is_empty() || request.credential.token.is_empty() {
            return Ok(UserAuthenticateResponse { accept: false });
        }
        let user: UserInfo = match self.db_client.get_user(&request.credential.id) {
            Ok(value) => value,
            Err(_) => return Ok(UserAuthenticateResponse { accept: false }),
        };
        Ok(UserAuthenticateResponse {
            accept: user.validate_token(&self.jwt_secret, &request.credential.token),
        })
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use crate::user_db::*;
    use crate::user_info::*;
    use std::vec;
    use teaclave_proto::teaclave_common::UserCredential;

    fn get_mock_service() -> TeaclaveAuthenticationService {
        let database = Database::open().unwrap();
        TeaclaveAuthenticationService::new(database.get_client()).unwrap()
    }

    pub fn test_user_register() {
        let request = UserRegisterRequest {
            id: "test_register_id".to_string(),
            password: "test_password".to_string(),
        };
        let request = Request::new(request);
        let service = get_mock_service();
        assert!(service.user_register(request).is_ok());
    }

    pub fn test_user_login() {
        let service = get_mock_service();
        let request = UserRegisterRequest {
            id: "test_login_id".to_string(),
            password: "test_password".to_string(),
        };
        let request = Request::new(request);
        assert!(service.user_register(request).is_ok());
        let request = UserLoginRequest {
            id: "test_login_id".to_string(),
            password: "test_password".to_string(),
        };
        let request = Request::new(request);
        assert!(service.user_login(request).is_ok());

        info!(
            "saved user_info: {:?}",
            service.db_client.get_user("test_login_id").unwrap()
        );
        let request = UserLoginRequest {
            id: "test_login_id".to_string(),
            password: "test_password1".to_string(),
        };
        let request = Request::new(request);
        assert!(service.user_login(request).is_err());
    }

    pub fn test_user_authenticate() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let request = UserRegisterRequest {
            id: id.to_string(),
            password: "test_password".to_string(),
        };
        let request = Request::new(request);
        assert!(service.user_register(request).is_ok());

        let request = UserLoginRequest {
            id: id.to_string(),
            password: "test_password".to_string(),
        };
        let request = Request::new(request);
        let token = service.user_login(request).unwrap().token;
        info!("login token: {}", token);
        dump_token(&service.jwt_secret, &token);

        let response = get_authenticate_response(id, &token, &service);
        assert!(response.accept);

        info!("test wrong algorithm");
        let my_claims = get_correct_claim();
        let token = gen_token(
            my_claims,
            Some(jsonwebtoken::Algorithm::HS256),
            &service.jwt_secret,
        );
        dump_token(&service.jwt_secret, &token);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);

        info!("test wrong issuer");
        let mut my_claims = get_correct_claim();
        my_claims.iss = "wrong issuer".to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        dump_token(&service.jwt_secret, &token);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);

        info!("test wrong user");
        let mut my_claims = get_correct_claim();
        my_claims.sub = "wrong user".to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        dump_token(&service.jwt_secret, &token);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);

        info!("test expired token");
        let mut my_claims = get_correct_claim();
        my_claims.exp -= 24 * 60 + 1;
        let token = gen_token(my_claims, None, &service.jwt_secret);
        dump_token(&service.jwt_secret, &token);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);

        info!("test wrong secret");
        let my_claims = get_correct_claim();
        let token = gen_token(my_claims, None, b"bad secret");
        dump_token(&service.jwt_secret, &token);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
    }

    fn get_correct_claim() -> Claims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Claims {
            sub: "test_authenticate_id".to_string(),
            iss: ISSUER_NAME.to_string(),
            exp: now + 24 * 60,
        }
    }

    fn gen_token(claim: Claims, bad_alg: Option<jsonwebtoken::Algorithm>, secret: &[u8]) -> String {
        let mut header = jsonwebtoken::Header::default();
        header.alg = bad_alg.unwrap_or(JWT_ALG);
        jsonwebtoken::encode(&header, &claim, secret).unwrap()
    }

    fn get_authenticate_response(
        id: &str,
        token: &str,
        service: &TeaclaveAuthenticationService,
    ) -> UserAuthenticateResponse {
        let credential = UserCredential {
            id: id.to_string(),
            token: token.to_string(),
        };
        let request = UserAuthenticateRequest { credential };
        let request = Request::new(request);
        service.user_authenticate(request).unwrap()
    }

    fn dump_token(secret: &[u8], token: &str) {
        let validation = jsonwebtoken::Validation {
            iss: Some(ISSUER_NAME.to_string()),
            sub: Some("test_authenticate_id".to_string()),
            algorithms: vec![JWT_ALG],
            ..Default::default()
        };
        let token_data =
            jsonwebtoken::decode::<crate::user_info::Claims>(token, secret, &validation);
        info!("token {:?}", token_data);
    }
}
