use anyhow::Result;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternalClient, UserAuthenticateRequest,
};
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    RegisterInputFileRequest, RegisterInputFileResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse, TeaclaveFrontend,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
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
    management_client: Arc<Mutex<TeaclaveManagementClient>>,
}

impl TeaclaveFrontendService {
    pub(crate) fn new(
        authentication_service_endpoint: Endpoint,
        management_service_endpoint: Endpoint,
    ) -> Result<Self> {
        let authentication_channel = authentication_service_endpoint.connect()?;
        let authentication_client = Arc::new(Mutex::new(
            TeaclaveAuthenticationInternalClient::new(authentication_channel)?,
        ));

        let management_channel = management_service_endpoint.connect()?;
        let management_client = Arc::new(Mutex::new(TeaclaveManagementClient::new(
            management_channel,
        )?));

        Ok(Self {
            authentication_client,
            management_client,
        })
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
        let response = self
            .management_client
            .clone()
            .lock()
            .unwrap()
            .register_input_file(request.message)?;
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
        let response = self
            .management_client
            .clone()
            .lock()
            .unwrap()
            .register_output_file(request.message)?;
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
        let credential = UserCredential::new(id, token);
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
