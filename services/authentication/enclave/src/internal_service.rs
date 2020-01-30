use crate::user_db::DbClient;
use crate::user_info::UserInfo;
use std::prelude::v1::*;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternal, UserAuthenticateRequest, UserAuthenticateResponse,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::TeaclaveServiceResponseResult;

#[teaclave_service(teaclave_authentication_service, TeaclaveAuthenticationInternal)]
#[derive(Clone)]
pub(crate) struct TeaclaveAuthenticationInternalService {
    pub(crate) db_client: DbClient,
    pub(crate) jwt_secret: Vec<u8>,
}

impl TeaclaveAuthenticationInternal for TeaclaveAuthenticationInternalService {
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
    use rand::RngCore;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use std::untrusted::time::SystemTimeEx;
    use std::vec;
    use teaclave_proto::teaclave_common::UserCredential;

    fn get_mock_service() -> TeaclaveAuthenticationInternalService {
        let database = Database::open().unwrap();
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        let user = UserInfo::new("test_authenticate_id", "test_authenticate_id");
        database.get_client().create_user(&user).unwrap();
        TeaclaveAuthenticationInternalService {
            db_client: database.get_client(),
            jwt_secret,
        }
    }

    pub fn test_user_authenticate() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let user = service.db_client.get_user(id).unwrap();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let exp = (now + Duration::from_secs(24 * 60)).as_secs();
        let token = user.get_token(exp, &service.jwt_secret).unwrap();

        let response = get_authenticate_response(id, &token, &service);
        assert!(response.accept);
        let token = validate_token(id, &service.jwt_secret, &token);
        info!("valid token: {:?}", token.unwrap());
    }

    pub fn test_invalid_algorithm() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let my_claims = get_correct_claim(id);
        let token = gen_token(
            my_claims,
            Some(jsonwebtoken::Algorithm::HS256),
            &service.jwt_secret,
        );
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm => (),
            _ => panic!("wrong error type"),
        }
    }

    pub fn test_invalid_issuer() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.iss = "wrong issuer".to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => (),
            _ => panic!("wrong error type"),
        }
    }

    pub fn test_expired_token() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.exp -= 24 * 60 + 1;
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => (),
            _ => panic!("wrong error type"),
        }
    }

    pub fn test_invalid_user() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.sub = "wrong user".to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSubject => (),
            _ => panic!("wrong error type"),
        }
    }

    pub fn test_wrong_secret() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let my_claims = get_correct_claim(id);
        let token = gen_token(my_claims, None, b"bad secret");
        let response = get_authenticate_response(id, &token, &service);
        assert!(!response.accept);
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => (),
            _ => panic!("wrong error type"),
        }
    }

    fn get_correct_claim(id: &str) -> Claims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Claims {
            sub: id.to_string(),
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
        service: &TeaclaveAuthenticationInternalService,
    ) -> UserAuthenticateResponse {
        let credential = UserCredential {
            id: id.to_string(),
            token: token.to_string(),
        };
        let request = UserAuthenticateRequest { credential };
        let request = Request::new(request);
        service.user_authenticate(request).unwrap()
    }

    fn validate_token(
        id: &str,
        secret: &[u8],
        token: &str,
    ) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<Claims>> {
        let validation = jsonwebtoken::Validation {
            iss: Some(ISSUER_NAME.to_string()),
            sub: Some(id.to_string()),
            algorithms: vec![JWT_ALG],
            ..Default::default()
        };
        jsonwebtoken::decode::<crate::user_info::Claims>(token, secret, &validation)
    }
}
