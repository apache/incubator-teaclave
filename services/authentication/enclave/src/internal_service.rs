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
use crate::user_db::DbClient;
use crate::user_info::UserInfo;
use std::sync::{Arc, Mutex};
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternal, UserAuthenticateRequest, UserAuthenticateResponse,
};
use teaclave_rpc::{ensure, Request, Response};
use teaclave_service_enclave_utils::bail;
use teaclave_types::TeaclaveServiceResponseResult;
#[derive(Clone)]
pub(crate) struct TeaclaveAuthenticationInternalService {
    db_client: Arc<Mutex<DbClient>>,
    jwt_secret: Vec<u8>,
}

impl TeaclaveAuthenticationInternalService {
    pub(crate) fn new(db_client: DbClient, jwt_secret: Vec<u8>) -> Self {
        Self {
            db_client: Arc::new(Mutex::new(db_client)),
            jwt_secret,
        }
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveAuthenticationInternal for TeaclaveAuthenticationInternalService {
    async fn user_authenticate(
        &self,
        request: Request<UserAuthenticateRequest>,
    ) -> TeaclaveServiceResponseResult<UserAuthenticateResponse> {
        let request = request.into_inner();
        ensure!(
            request.credential.is_some(),
            AuthenticationError::IncorrectToken
        );
        let cred = request.credential.unwrap();
        ensure!(!cred.id.is_empty(), AuthenticationError::InvalidUserId);
        ensure!(!cred.token.is_empty(), AuthenticationError::InvalidToken);
        let user: UserInfo = match self.db_client.lock().unwrap().get_user(&cred.id) {
            Ok(value) => value,
            Err(_) => bail!(AuthenticationError::InvalidUserId),
        };
        let claims = user
            .validate_token(&self.jwt_secret, &cred.token)
            .map_err(|_| AuthenticationError::IncorrectToken)?;
        Ok(Response::new(UserAuthenticateResponse::new(claims)))
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use crate::user_db::*;
    use crate::user_info::*;
    use rand::RngCore;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    #[allow(unused_imports)]
    use std::untrusted::time::SystemTimeEx;
    use std::vec;
    use teaclave_proto::teaclave_common::UserCredential;
    use teaclave_rpc::IntoRequest;
    use teaclave_types::{UserAuthClaims, UserRole};

    fn get_mock_service() -> TeaclaveAuthenticationInternalService {
        let database = Database::open("").unwrap();
        let mut jwt_secret = vec![0; JWT_SECRET_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut jwt_secret);
        let user = UserInfo::new(
            "test_authenticate_id",
            "test_authenticate_id",
            UserRole::PlatformAdmin,
        );
        database.get_client().create_user(&user).unwrap();
        TeaclaveAuthenticationInternalService {
            db_client: Arc::new(Mutex::new(database.get_client())),
            jwt_secret,
        }
    }

    pub async fn test_user_authenticate() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let user = service.db_client.lock().unwrap().get_user(id).unwrap();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let exp = (now + Duration::from_secs(24 * 60 * 60)).as_secs(); // 1 day
        let token = user.get_token(exp, &service.jwt_secret).unwrap();

        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_ok());
        let token = validate_token(id, &service.jwt_secret, &token);
        debug!("valid token: {:?}", token.unwrap());
    }

    pub async fn test_invalid_algorithm() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let my_claims = get_correct_claim(id);
        let token = gen_token(
            my_claims,
            Some(jsonwebtoken::Algorithm::HS256),
            &service.jwt_secret,
        );
        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_err());
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm => (),
            _ => panic!("wrong error type"),
        }
    }

    pub async fn test_invalid_issuer() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.iss = "wrong issuer".to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_err());
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => (),
            _ => panic!("wrong error type"),
        }
    }

    pub async fn test_expired_token() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.exp -= 24 * 60 + 1;
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_err());
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => (),
            _ => panic!("wrong error type"),
        }
    }

    pub async fn test_invalid_user() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let mut my_claims = get_correct_claim(id);
        my_claims.sub = "wrong user".to_string();
        my_claims.role = UserRole::PlatformAdmin.to_string();
        let token = gen_token(my_claims, None, &service.jwt_secret);
        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_err());
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSubject => (),
            _ => panic!("wrong error type"),
        }
    }

    pub async fn test_wrong_secret() {
        let id = "test_authenticate_id";
        let service = get_mock_service();
        let my_claims = get_correct_claim(id);
        let token = gen_token(my_claims, None, b"bad secret");
        let response = get_authenticate_response(id, &token, &service).await;
        assert!(response.is_err());
        let error = validate_token(id, &service.jwt_secret, &token);
        assert!(error.is_err());
        match *error.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::InvalidSignature => (),
            _ => panic!("wrong error type"),
        }
    }

    fn get_correct_claim(id: &str) -> UserAuthClaims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        UserAuthClaims {
            sub: id.to_owned(),
            role: UserRole::PlatformAdmin.to_string(),
            iss: ISSUER_NAME.to_string(),
            exp: now + 24 * 60,
        }
    }

    fn gen_token(
        claim: UserAuthClaims,
        bad_alg: Option<jsonwebtoken::Algorithm>,
        secret: &[u8],
    ) -> String {
        let header = jsonwebtoken::Header {
            alg: bad_alg.unwrap_or(JWT_ALG),
            ..Default::default()
        };
        let secret = jsonwebtoken::EncodingKey::from_secret(secret);
        jsonwebtoken::encode(&header, &claim, &secret).unwrap()
    }

    async fn get_authenticate_response(
        id: &str,
        token: &str,
        service: &TeaclaveAuthenticationInternalService,
    ) -> TeaclaveServiceResponseResult<UserAuthenticateResponse> {
        let credential = UserCredential::new(id, token);
        let request = UserAuthenticateRequest::new(credential).into_request();
        service.user_authenticate(request).await
    }

    fn validate_token(
        id: &str,
        secret: &[u8],
        token: &str,
    ) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<UserAuthClaims>> {
        let validation = jsonwebtoken::Validation {
            iss: Some(ISSUER_NAME.to_string()),
            sub: Some(id.to_string()),
            algorithms: vec![JWT_ALG],
            ..Default::default()
        };
        let secret = jsonwebtoken::DecodingKey::from_secret(secret);
        jsonwebtoken::decode::<UserAuthClaims>(token, &secret, &validation)
    }
}
