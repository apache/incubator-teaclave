#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AllocDataRequest {
    #[prost(enumeration="kms_proto::proto::EncType", required, tag="1")]
    pub enc_type: i32,
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AllocDataResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputRequest {
    #[prost(enumeration="kms_proto::proto::EncType", required, tag="1")]
    pub enc_type: i32,
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    #[prost(message, required, tag="3")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadDataResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// SharedOutput will have muliple owners
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// Output data doesn't have meta
    #[prost(message, optional, tag="3")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// SharedOutput doesn't have storage info 
    #[prost(message, optional, tag="4")]
    pub storage_info: ::std::option::Option<tdfs_common_proto::data_common_proto::DataStorageInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TDFSExternalRequest {
    AllocData(AllocDataRequest),
    RegisterSharedOutput(RegisterSharedOutputRequest),
    ReadData(ReadDataRequest),
    SaveData(SaveDataRequest),
    DeleteData(DeleteDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TDFSExternalResponse {
    AllocData(AllocDataResponse),
    RegisterSharedOutput(RegisterSharedOutputResponse),
    ReadData(ReadDataResponse),
    SaveData(SaveDataResponse),
    DeleteData(DeleteDataResponse),
}
pub trait TDFSExternalService {
    fn alloc_data(req: AllocDataRequest) -> mesatee_core::Result<AllocDataResponse>;
    fn register_shared_output(req: RegisterSharedOutputRequest) -> mesatee_core::Result<RegisterSharedOutputResponse>;
    fn read_data(req: ReadDataRequest) -> mesatee_core::Result<ReadDataResponse>;
    fn save_data(req: SaveDataRequest) -> mesatee_core::Result<SaveDataResponse>;
    fn delete_data(req: DeleteDataRequest) -> mesatee_core::Result<DeleteDataResponse>;
    fn dispatch(&self, req: TDFSExternalRequest) -> mesatee_core::Result<TDFSExternalResponse> {
        let authenticated = match req {
            TDFSExternalRequest::AllocData(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::RegisterSharedOutput(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::ReadData(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::SaveData(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::DeleteData(ref req) => req.creds.get_creds().auth(),
        };
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
        match req {
            TDFSExternalRequest::AllocData(req) => Self::alloc_data(req).map(TDFSExternalResponse::AllocData),            TDFSExternalRequest::RegisterSharedOutput(req) => Self::register_shared_output(req).map(TDFSExternalResponse::RegisterSharedOutput),            TDFSExternalRequest::ReadData(req) => Self::read_data(req).map(TDFSExternalResponse::ReadData),            TDFSExternalRequest::SaveData(req) => Self::save_data(req).map(TDFSExternalResponse::SaveData),            TDFSExternalRequest::DeleteData(req) => Self::delete_data(req).map(TDFSExternalResponse::DeleteData),        }
    }
}
pub struct TDFSExternalClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<TDFSExternalRequest, TDFSExternalResponse>,
}

impl TDFSExternalClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<TDFSExternalRequest, TDFSExternalResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(TDFSExternalClient { channel })
    }
}
impl TDFSExternalClient {
    pub fn alloc_data(&mut self, req: AllocDataRequest) -> mesatee_core::Result<AllocDataResponse> {
        let req = TDFSExternalRequest::AllocData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::AllocData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn register_shared_output(&mut self, req: RegisterSharedOutputRequest) -> mesatee_core::Result<RegisterSharedOutputResponse> {
        let req = TDFSExternalRequest::RegisterSharedOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::RegisterSharedOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn read_data(&mut self, req: ReadDataRequest) -> mesatee_core::Result<ReadDataResponse> {
        let req = TDFSExternalRequest::ReadData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::ReadData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn save_data(&mut self, req: SaveDataRequest) -> mesatee_core::Result<SaveDataResponse> {
        let req = TDFSExternalRequest::SaveData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::SaveData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn delete_data(&mut self, req: DeleteDataRequest) -> mesatee_core::Result<DeleteDataResponse> {
        let req = TDFSExternalRequest::DeleteData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::DeleteData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
