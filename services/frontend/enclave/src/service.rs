#![allow(dead_code)]

use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend_service::{
    RegisterInputFileRequest, RegisterInputFileResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse, TeaclaveFrontend,
};
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
pub(crate) struct TeaclaveFrontendService;

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn register_input_file(
        &self,
        _request: RegisterInputFileRequest,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        unimplemented!()
    }

    fn register_output_file(
        &self,
        _request: RegisterOutputFileRequest,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        unimplemented!()
    }
}
