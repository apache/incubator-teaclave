use crate::service::TeaclaveDatabaseError;
use std::prelude::v1::*;
use std::sync::mpsc::{channel, Sender};
use teaclave_proto::teaclave_database_service::{
    TeaclaveDatabaseRequest, TeaclaveDatabaseResponse,
};
use teaclave_rpc::Request;
use teaclave_types::TeaclaveServiceResponseResult;

#[derive(Clone)]
pub(crate) struct ProxyService {
    sender: Sender<ProxyRequest>,
}

impl ProxyService {
    pub(crate) fn new(sender: Sender<ProxyRequest>) -> Self {
        Self { sender }
    }
}

impl teaclave_rpc::TeaclaveService<TeaclaveDatabaseRequest, TeaclaveDatabaseResponse>
    for ProxyService
{
    fn handle_request(
        &self,
        request: Request<TeaclaveDatabaseRequest>,
    ) -> TeaclaveServiceResponseResult<TeaclaveDatabaseResponse> {
        let (sender, receiver) = channel();
        self.sender
            .send(ProxyRequest { sender, request })
            .map_err(|_| TeaclaveDatabaseError::MpscError)?;
        receiver
            .recv()
            .map_err(|_| TeaclaveDatabaseError::MpscError)?
    }
}

#[derive(Clone)]
pub(crate) struct ProxyRequest {
    pub sender: Sender<TeaclaveServiceResponseResult<TeaclaveDatabaseResponse>>,
    pub request: Request<TeaclaveDatabaseRequest>,
}
