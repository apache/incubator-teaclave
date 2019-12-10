/// Request for saving Output
/// Update the meta information of the data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct SaveDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Request for accessing data
/// TMS can use the request to prepare inputs and outputs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AccessDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for accessing data
/// The response includes the storage_info and key_config, 
/// so the worker can read/write the data.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AccessDataResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// SharedOutput will have muliple owners
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// Output data doesn't have meta
    #[prost(message, optional, tag="3")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// SharedOutput have internal storage info 
    #[prost(message, required, tag="4")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// The config for encryption/decryption
    #[prost(message, required, tag="5")]
    pub config: kms_proto::proto::KeyConfig,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TDFSInternalRequest {
    AccessData(AccessDataRequest),
    SaveData(SaveDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TDFSInternalResponse {
    AccessData(AccessDataResponse),
    SaveData(SaveDataResponse),
}
pub trait TDFSInternalService {
    fn access_data(req: AccessDataRequest) -> mesatee_core::Result<AccessDataResponse>;
    fn save_data(req: SaveDataRequest) -> mesatee_core::Result<SaveDataResponse>;
    fn dispatch(&self, req: TDFSInternalRequest) -> mesatee_core::Result<TDFSInternalResponse> {
        match req {
            TDFSInternalRequest::AccessData(req) => Self::access_data(req).map(TDFSInternalResponse::AccessData),            TDFSInternalRequest::SaveData(req) => Self::save_data(req).map(TDFSInternalResponse::SaveData),        }
    }
}
pub struct TDFSInternalClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<TDFSInternalRequest, TDFSInternalResponse>,
}

impl TDFSInternalClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<TDFSInternalRequest, TDFSInternalResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(TDFSInternalClient { channel })
    }
}
impl TDFSInternalClient {
    pub fn access_data(&mut self, req: AccessDataRequest) -> mesatee_core::Result<AccessDataResponse> {
        let req = TDFSInternalRequest::AccessData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::AccessData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn save_data(&mut self, req: SaveDataRequest) -> mesatee_core::Result<SaveDataResponse> {
        let req = TDFSInternalRequest::SaveData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::SaveData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
