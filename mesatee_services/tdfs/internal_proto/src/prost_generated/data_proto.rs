/// Request for writing output
/// Update the meta information of the output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct WriteOutputRequest {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// meta information of the data
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
}
/// Response for writing output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct WriteOutputResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Request for fetching input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchInputRequest {
    /// data indentifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for fetching input
/// TMS can use the request to prepare inputs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchInputResponse {
    /// meta information of the input, for checking integrity
    #[prost(message, required, tag="1")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    /// storage information for reading the data
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// key config for file decryption
    #[prost(message, required, tag="3")]
    pub config: kms_proto::proto::KeyConfig,
}
/// Request for fetching output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchOutputRequest {
    /// data indentifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for fetching output
/// TMS can use the request to prepare outputs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchOutputResponse {
    /// storage information for writing the data
    #[prost(message, required, tag="1")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// key config for file encryption
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
}
/// Request for registing shared output (data fusion)
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputRequest {
    /// owners of fused data 
    #[prost(string, repeated, tag="1")]
    pub owners: ::std::vec::Vec<std::string::String>,
}
/// Response for registing shared output
/// The response includes the storage_info and key_config, 
/// so the worker can read/write the data.
/// Data id should be returned to the task participants.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterSharedOutputResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// internal storage info for writing output
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// key config for file encryption
    #[prost(message, required, tag="3")]
    pub config: kms_proto::proto::KeyConfig,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TDFSInternalRequest {
    FetchInput(FetchInputRequest),
    FetchOutput(FetchOutputRequest),
    RegisterSharedOutput(RegisterSharedOutputRequest),
    WriteOutput(WriteOutputRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TDFSInternalResponse {
    FetchInput(FetchInputResponse),
    FetchOutput(FetchOutputResponse),
    RegisterSharedOutput(RegisterSharedOutputResponse),
    WriteOutput(WriteOutputResponse),
}
pub trait TDFSInternalService {
    fn fetch_input(req: FetchInputRequest) -> mesatee_core::Result<FetchInputResponse>;
    fn fetch_output(req: FetchOutputRequest) -> mesatee_core::Result<FetchOutputResponse>;
    fn register_shared_output(req: RegisterSharedOutputRequest) -> mesatee_core::Result<RegisterSharedOutputResponse>;
    fn write_output(req: WriteOutputRequest) -> mesatee_core::Result<WriteOutputResponse>;
    fn dispatch(&self, req: TDFSInternalRequest) -> mesatee_core::Result<TDFSInternalResponse> {
        match req {
            TDFSInternalRequest::FetchInput(req) => Self::fetch_input(req).map(TDFSInternalResponse::FetchInput),            TDFSInternalRequest::FetchOutput(req) => Self::fetch_output(req).map(TDFSInternalResponse::FetchOutput),            TDFSInternalRequest::RegisterSharedOutput(req) => Self::register_shared_output(req).map(TDFSInternalResponse::RegisterSharedOutput),            TDFSInternalRequest::WriteOutput(req) => Self::write_output(req).map(TDFSInternalResponse::WriteOutput),        }
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
    pub fn fetch_input(&mut self, req: FetchInputRequest) -> mesatee_core::Result<FetchInputResponse> {
        let req = TDFSInternalRequest::FetchInput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::FetchInput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn fetch_output(&mut self, req: FetchOutputRequest) -> mesatee_core::Result<FetchOutputResponse> {
        let req = TDFSInternalRequest::FetchOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::FetchOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn register_shared_output(&mut self, req: RegisterSharedOutputRequest) -> mesatee_core::Result<RegisterSharedOutputResponse> {
        let req = TDFSInternalRequest::RegisterSharedOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::RegisterSharedOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn write_output(&mut self, req: WriteOutputRequest) -> mesatee_core::Result<WriteOutputResponse> {
        let req = TDFSInternalRequest::WriteOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSInternalResponse::WriteOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
