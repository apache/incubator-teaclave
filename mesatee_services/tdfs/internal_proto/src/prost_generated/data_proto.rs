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
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadDataRequest {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
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
    /// SharedOutput have internal storage info 
    #[prost(message, required, tag="4")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TDFSInternalRequest {
    ReadData(ReadDataRequest),
    SaveData(SaveDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TDFSInternalResponse {
    ReadData(ReadDataResponse),
    SaveData(SaveDataResponse),
}
pub trait TDFSInternalService {
    fn read_data(req: ReadDataRequest) -> mesatee_core::Result<ReadDataResponse>;
    fn save_data(req: SaveDataRequest) -> mesatee_core::Result<SaveDataResponse>;
    fn dispatch(&self, req: TDFSInternalRequest) -> mesatee_core::Result<TDFSInternalResponse> {
        match req {
            TDFSInternalRequest::ReadData(req) => Self::read_data(req).map(TDFSInternalResponse::ReadData),            TDFSInternalRequest::SaveData(req) => Self::save_data(req).map(TDFSInternalResponse::SaveData),        }
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
    pub fn read_data(&mut self, req: ReadDataRequest) -> mesatee_core::Result<ReadDataResponse> {
        let req = TDFSInternalRequest::ReadData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::ReadData(resp) => Ok(resp),
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
