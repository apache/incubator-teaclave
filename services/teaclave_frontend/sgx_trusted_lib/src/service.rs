use core::convert::Into;
use core::convert::TryFrom;
use proto::TeaclaveFrontend;
use std::prelude::v1::*;
use teaclave_frontend_proto::*;
use teaclave_rpc::TeaclaveService;

pub(crate) struct TeaclaveFrontendService;

impl proto::TeaclaveFrontend for TeaclaveFrontendService {
    fn user_login(req: proto::UserLoginRequest) -> anyhow::Result<proto::UserLoginResponse> {
        let request = UserLoginRequest::try_from(req)?;
        let response = UserLoginResponse {
            token: "test_token".to_string(),
        }
        .into();
        Ok(response)
    }
}

impl TeaclaveService<proto::TeaclaveFrontendRequest, proto::TeaclaveFrontendResponse>
    for TeaclaveFrontendService
{
    fn handle_request(
        &self,
        request: proto::TeaclaveFrontendRequest,
    ) -> anyhow::Result<proto::TeaclaveFrontendResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", request);
        self.dispatch(request)
    }
}
