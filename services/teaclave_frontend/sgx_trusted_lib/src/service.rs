use std::prelude::v1::*;
use teaclave_frontend_proto::{TeaclaveFrontend, UserLoginRequest, UserLoginResponse};
use teaclave_service_sgx_utils::teaclave_service;

#[teaclave_service(teaclave_frontend_proto, TeaclaveFrontend)]
pub(crate) struct TeaclaveFrontendService;

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn user_login(_request: UserLoginRequest) -> anyhow::Result<UserLoginResponse> {
        let response = UserLoginResponse {
            token: "test_token".to_string(),
        };
        Ok(response)
    }
}
