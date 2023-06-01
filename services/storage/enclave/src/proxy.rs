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

use crate::error::StorageServiceError;
use anyhow::anyhow;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::{Request, Response, Status};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[derive(Clone)]
pub(crate) struct ProxyService {
    sender: UnboundedSender<ProxyRequest>,
}

impl ProxyService {
    pub(crate) fn new(sender: UnboundedSender<ProxyRequest>) -> Self {
        Self { sender }
    }
}

macro_rules! send_request {
    ($service: ident,$request:expr,$fun:ident,$response:ident) => {{
        let (sender, mut receiver) = unbounded_channel();
        let request = $request.into_inner();
        $service
            .sender
            .send(ProxyRequest {
                sender,
                request: Request::new(TeaclaveStorageRequest::$fun(request)),
            })
            .map_err(|_| StorageServiceError::Service(anyhow!("send ProxyRequest error")))?;
        match receiver.recv().await {
            Some(Ok(TeaclaveStorageResponse::$response(re))) => return Ok(Response::new(re)),
            _ => return Err(teaclave_rpc::Status::internal("invalid response")),
        }
    }};
}

#[teaclave_rpc::async_trait]
impl TeaclaveStorage for ProxyService {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        send_request!(self, request, Get, Get)
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<()>, Status> {
        send_request!(self, request, Put, Empty)
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        send_request!(self, request, Delete, Empty)
    }

    async fn enqueue(&self, request: Request<EnqueueRequest>) -> Result<Response<()>, Status> {
        send_request!(self, request, Enqueue, Empty)
    }

    async fn dequeue(
        &self,
        request: Request<DequeueRequest>,
    ) -> Result<Response<DequeueResponse>, Status> {
        send_request!(self, request, Dequeue, Dequeue)
    }

    async fn get_keys_by_prefix(
        &self,
        request: Request<GetKeysByPrefixRequest>,
    ) -> Result<Response<GetKeysByPrefixResponse>, Status> {
        send_request!(self, request, GetKeysByPrefix, GetKeysByPrefix)
    }
}

pub(crate) struct ProxyRequest {
    pub sender: UnboundedSender<std::result::Result<TeaclaveStorageResponse, StorageServiceError>>,
    pub request: Request<TeaclaveStorageRequest>,
}
