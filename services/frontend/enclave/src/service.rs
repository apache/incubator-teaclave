#![allow(dead_code)]

use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_authentication_service::TeaclaveAuthenticationClient;
use teaclave_proto::teaclave_authentication_service::UserAuthenticateRequest;
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    RegisterInputFileRequest, RegisterInputFileResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse, TeaclaveFrontend,
};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclaveFrontendError {
    #[error("authentication error")]
    AuthenticationError,
}

impl From<TeaclaveFrontendError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveFrontendError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_frontend_service, TeaclaveFrontend, TeaclaveFrontendError)]
#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationClient>>,
}

impl TeaclaveFrontendService {
    pub(crate) fn new(config: &RuntimeConfig) -> Self {
        let channel = Endpoint::new(&config.internal_endpoints.authentication.advertised_address)
            .connect()
            .unwrap();
        let client = TeaclaveAuthenticationClient::new(channel).unwrap();
        Self {
            authentication_client: Arc::new(Mutex::new(client)),
        }
    }
}

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn register_input_file(
        &self,
        request: RegisterInputFileRequest,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        if !self.authenticate(request.credential) {
            return Err(TeaclaveFrontendError::AuthenticationError.into());
        }
        let response = RegisterInputFileResponse {
            data_id: "".to_string(),
        };
        Ok(response)
    }

    fn register_output_file(
        &self,
        request: RegisterOutputFileRequest,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        if !self.authenticate(request.credential) {
            return Err(TeaclaveFrontendError::AuthenticationError.into());
        }
        let response = RegisterOutputFileResponse {
            data_id: "".to_string(),
        };
        Ok(response)
    }
}

impl TeaclaveFrontendService {
    fn authenticate(&self, credential: UserCredential) -> bool {
        let auth_request = UserAuthenticateRequest { credential };
        let auth_response = self
            .authentication_client
            .clone()
            .lock()
            .unwrap()
            .user_authenticate(auth_request);
        auth_response.unwrap().accept
    }
}
