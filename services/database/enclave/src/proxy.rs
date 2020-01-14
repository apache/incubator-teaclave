use std::prelude::v1::*;
use teaclave_proto::teaclave_database_service::{
    TeaclaveDatabaseRequest, TeaclaveDatabaseResponse
};
use teaclave_types::TeaclaveServiceResponseResult;
use std::sync::mpsc::{Sender, channel};
use crate::service::TeaclaveDatabaseError;

#[derive(Clone)]
pub struct ProxyService {
    pub sender: Sender<ProxyRequest>,
}

impl teaclave_rpc::TeaclaveService<TeaclaveDatabaseRequest, TeaclaveDatabaseResponse> for ProxyService {
    fn handle_request(
        &self,
        request: TeaclaveDatabaseRequest,
    ) -> TeaclaveServiceResponseResult<TeaclaveDatabaseResponse> {
        let (sender, receiver) = channel();
        self.sender.send(ProxyRequest {
            sender,
            request,
        }).map_err(|_| TeaclaveDatabaseError::MpscError)?;
        receiver.recv().map_err(|_| TeaclaveDatabaseError::MpscError)?
    }
}

#[derive(Clone)]
pub struct ProxyRequest {
    pub sender: Sender<TeaclaveServiceResponseResult<TeaclaveDatabaseResponse>>,
    pub request: TeaclaveDatabaseRequest,
}
