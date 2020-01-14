use anyhow::{Error, Result};
use std::prelude::v1::*;

pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        "/teaclave_database_service_proto.rs"
    ));
}

pub use proto::TeaclaveDatabase;
pub use proto::TeaclaveDatabaseClient;
pub use proto::TeaclaveDatabaseRequest;
pub use proto::TeaclaveDatabaseResponse;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct GetRequest {
    pub key: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct GetResponse {
    pub value: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct PutRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct PutResponse { }

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct DeleteRequest {
    pub key: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct DeleteResponse { }

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct EnqueueRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct EnqueueResponse { }

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct DequeueRequest {
    pub key: Vec<u8>,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct DequeueResponse { 
    pub value: Vec<u8>,
}

impl std::convert::TryFrom<proto::GetRequest> for GetRequest {
    type Error = Error;

    fn try_from(proto: proto::GetRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key
        };

        Ok(ret)
    }
}

impl From<GetRequest> for proto::GetRequest {
    fn from(request: GetRequest) -> Self {
        Self {
            key: request.key,
        }
    }
}


impl std::convert::TryFrom<proto::GetResponse> for GetResponse {
    type Error = Error;

    fn try_from(proto: proto::GetResponse) -> Result<Self> {
        let ret = Self {
            value: proto.value
        };

        Ok(ret)
    }
}

impl From<GetResponse> for proto::GetResponse {
    fn from(request: GetResponse) -> Self {
        Self {
            value: request.value,
        }
    }
}

impl std::convert::TryFrom<proto::PutRequest> for PutRequest {
    type Error = Error;

    fn try_from(proto: proto::PutRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
            value: proto.value,
        };

        Ok(ret)
    }
}

impl From<PutRequest> for proto::PutRequest {
    fn from(request: PutRequest) -> Self {
        Self {
            key: request.key,
            value: request.value,
        }
    }
}

impl std::convert::TryFrom<proto::PutResponse> for PutResponse {
    type Error = Error;

    fn try_from(_proto: proto::PutResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<PutResponse> for proto::PutResponse {
    fn from(_request: PutResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::DeleteRequest> for DeleteRequest {
    type Error = Error;

    fn try_from(proto: proto::DeleteRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
        };

        Ok(ret)
    }
}

impl From<DeleteRequest> for proto::DeleteRequest {
    fn from(request: DeleteRequest) -> Self {
        Self {
            key: request.key,
        }
    }
}

impl std::convert::TryFrom<proto::DeleteResponse> for DeleteResponse {
    type Error = Error;

    fn try_from(_proto: proto::DeleteResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<DeleteResponse> for proto::DeleteResponse {
    fn from(_request: DeleteResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::EnqueueRequest> for EnqueueRequest {
    type Error = Error;

    fn try_from(proto: proto::EnqueueRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
            value: proto.value,
        };

        Ok(ret)
    }
}

impl From<EnqueueRequest> for proto::EnqueueRequest {
    fn from(request: EnqueueRequest) -> Self {
        Self {
            key: request.key,
            value: request.value,
        }
    }
}

impl std::convert::TryFrom<proto::EnqueueResponse> for EnqueueResponse {
    type Error = Error;

    fn try_from(_proto: proto::EnqueueResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<EnqueueResponse> for proto::EnqueueResponse {
    fn from(_request: EnqueueResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::DequeueRequest> for DequeueRequest {
    type Error = Error;

    fn try_from(proto: proto::DequeueRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
        };

        Ok(ret)
    }
}

impl From<DequeueRequest> for proto::DequeueRequest {
    fn from(request: DequeueRequest) -> Self {
        Self {
            key: request.key,
        }
    }
}

impl std::convert::TryFrom<proto::DequeueResponse> for DequeueResponse {
    type Error = Error;

    fn try_from(proto: proto::DequeueResponse) -> Result<Self> {
        Ok(Self {
            value: proto.value,
        })
    }
}

impl From<DequeueResponse> for proto::DequeueResponse {
    fn from(request: DequeueResponse) -> Self {
        Self {
            value: request.value,
        }
    }
}