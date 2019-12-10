/// Request for allocing a data. 
/// The data can be used as input or output.
/// And we need this request to obtain a config for file encryption
/// Storage info is needed, as the data may be used as output.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AllocDataRequest {
    /// Encryption type: AEAD, ProtectedFS
    #[prost(enumeration="kms_proto::proto::EncType", required, tag="1")]
    pub enc_type: i32,
    /// Where to save/read the data
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for allocing a data
/// Config is used for file encryption
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AllocDataResponse {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// Key config: AEAD, ProtectedFS
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
}
/// Request for registering a shared output
/// The output can be used for data fusion
/// No need to provide a storage_info, it will be saved into internal storage.
/// Only platform knows the storage info and can access the data.
/// If result is saved to this data, it can be used as input in other tasks
/// But all the owners need to agree the task
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputRequest {
    /// Encryption type: AEAD, ProtectedFS
    #[prost(enumeration="kms_proto::proto::EncType", required, tag="1")]
    pub enc_type: i32,
    /// muliple owners
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for registering a shared output
/// Key config will not be returned
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Request for saving data.
/// Update the meta information so the data can be used as input
/// After this request, the data is fixed and can't be changed. 
/// And platform can use storage info and key config to read the data.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// meta information
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for saving data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Request for reading data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for reading data
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
    /// Storage info of SharedOutput will not be returned.
    #[prost(message, optional, tag="4")]
    pub storage_info: ::std::option::Option<tdfs_common_proto::data_common_proto::DataStorageInfo>,
    /// The key config of SharedOutput is only visible to platform
    #[prost(message, optional, tag="5")]
    pub config: ::std::option::Option<kms_proto::proto::KeyConfig>,
}
/// Request for deleting a data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for deleting a data
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
