use crate::service::TeaclaveStorageError;
use std::prelude::v1::*;
use std::sync::mpsc::{channel, Sender};
use teaclave_proto::teaclave_storage_service::{TeaclaveStorageRequest, TeaclaveStorageResponse};
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

impl teaclave_rpc::TeaclaveService<TeaclaveStorageRequest, TeaclaveStorageResponse>
    for ProxyService
{
    fn handle_request(
        &self,
        request: Request<TeaclaveStorageRequest>,
    ) -> TeaclaveServiceResponseResult<TeaclaveStorageResponse> {
        let (sender, receiver) = channel();
        self.sender
            .send(ProxyRequest { sender, request })
            .map_err(|_| TeaclaveStorageError::Connection)?;
        receiver
            .recv()
            .map_err(|_| TeaclaveStorageError::Connection)?
    }
}

#[derive(Clone)]
pub(crate) struct ProxyRequest {
    pub sender: Sender<TeaclaveServiceResponseResult<TeaclaveStorageResponse>>,
    pub request: Request<TeaclaveStorageRequest>,
}
