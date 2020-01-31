use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_authentication_service::TeaclaveAuthenticationInternalClient;
use teaclave_proto::teaclave_authentication_service::UserAuthenticateRequest;
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    RegisterInputFileRequest, RegisterInputFileResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse, TeaclaveFrontend,
};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
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
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient>>,
}

impl TeaclaveFrontendService {
    pub(crate) fn new(config: &RuntimeConfig) -> Self {
        let channel = Endpoint::new(&config.internal_endpoints.authentication.advertised_address)
            .connect()
            .unwrap();
        let client = TeaclaveAuthenticationInternalClient::new(channel).unwrap();
        Self {
            authentication_client: Arc::new(Mutex::new(client)),
        }
    }
}

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        match self.authenticate(&request) {
            Ok(true) => (),
            _ => return Err(TeaclaveFrontendError::AuthenticationError.into()),
        }
        let response = RegisterInputFileResponse {
            data_id: "".to_string(),
        };
        Ok(response)
    }

    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        match self.authenticate(&request) {
            Ok(true) => (),
            _ => return Err(TeaclaveFrontendError::AuthenticationError.into()),
        }
        let response = RegisterOutputFileResponse {
            data_id: "".to_string(),
        };
        Ok(response)
    }
}

impl TeaclaveFrontendService {
    fn authenticate<T>(&self, request: &Request<T>) -> anyhow::Result<bool> {
        use anyhow::anyhow;
        let id = request
            .metadata
            .get("id")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let token = request
            .metadata
            .get("token")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let credential = UserCredential::new(&id, &token);
        let auth_request = UserAuthenticateRequest { credential };
        let auth_response = self
            .authentication_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock authentication client"))?
            .user_authenticate(auth_request);
        Ok(auth_response?.accept)
    }
}
