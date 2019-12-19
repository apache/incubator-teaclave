/// Request for writing a private output
/// Update the meta information for the output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct WritePrivateOutputRequest {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// meta information of the data
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
}
/// Response for writing a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct WritePrivateOutputResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Request for fetching a private input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchPrivateInputRequest {
    /// data indentifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for fetching a private input
/// TMS can use the request to prepare inputs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchPrivateInputResponse {
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
/// Request for fetching a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchPrivateOutputRequest {
    /// data indentifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for fetching a private output
/// TMS can use the request to prepare outputs
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchPrivateOutputResponse {
    /// storage information for writing the data
    #[prost(message, required, tag="1")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// key config for file encryption
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
    /// whether the output is already written by a task
    #[prost(bool, required, tag="3")]
    pub is_complete: bool,
}
/// Request for registering fusion output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterFusionOutputRequest {
    /// owners of fused data 
    #[prost(string, repeated, tag="1")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// source task id
    #[prost(string, required, tag="2")]
    pub task_id: std::string::String,
}
/// Response for registing fusion output
/// The response includes the storage_info and key_config, 
/// so the worker can read/write the data.
/// Data id should be returned to the task participants.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterFusionOutputResponse {
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// internal storage info for writing output
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// key config for file encryption
    #[prost(message, required, tag="3")]
    pub config: kms_proto::proto::KeyConfig,
}
/// Request for feching a fusion data as an input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchFusionDataRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Response for fetching a fusion data as an input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchFusionDataResponse {
    /// Meta information. If output is not written, the meta infomation is None. 
    #[prost(message, optional, tag="1")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// key config for file decryption
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
    /// Owners of the data
    #[prost(string, repeated, tag="3")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// Which task generates the fustion data
    #[prost(string, required, tag="4")]
    pub source_task_id: std::string::String,
    /// Whether the output has been written by a task. 
    #[prost(bool, required, tag="5")]
    pub is_complete: bool,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum DataStoreServiceRequest {
    WritePrivateOutput(WritePrivateOutputRequest),
    FetchPrivateInput(FetchPrivateInputRequest),
    FetchPrivateOutput(FetchPrivateOutputRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DataStoreServiceResponse {
    WritePrivateOutput(WritePrivateOutputResponse),
    FetchPrivateInput(FetchPrivateInputResponse),
    FetchPrivateOutput(FetchPrivateOutputResponse),
}
pub trait DataStoreServiceService {
    fn write_private_output(req: WritePrivateOutputRequest) -> mesatee_core::Result<WritePrivateOutputResponse>;
    fn fetch_private_input(req: FetchPrivateInputRequest) -> mesatee_core::Result<FetchPrivateInputResponse>;
    fn fetch_private_output(req: FetchPrivateOutputRequest) -> mesatee_core::Result<FetchPrivateOutputResponse>;
    fn dispatch(&self, req: DataStoreServiceRequest) -> mesatee_core::Result<DataStoreServiceResponse> {
        match req {
            DataStoreServiceRequest::WritePrivateOutput(req) => Self::write_private_output(req).map(DataStoreServiceResponse::WritePrivateOutput),            DataStoreServiceRequest::FetchPrivateInput(req) => Self::fetch_private_input(req).map(DataStoreServiceResponse::FetchPrivateInput),            DataStoreServiceRequest::FetchPrivateOutput(req) => Self::fetch_private_output(req).map(DataStoreServiceResponse::FetchPrivateOutput),        }
    }
}
pub struct DataStoreServiceClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<DataStoreServiceRequest, DataStoreServiceResponse>,
}

impl DataStoreServiceClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<DataStoreServiceRequest, DataStoreServiceResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(DataStoreServiceClient { channel })
    }
}
impl DataStoreServiceClient {
    pub fn write_private_output(&mut self, req: WritePrivateOutputRequest) -> mesatee_core::Result<WritePrivateOutputResponse> {
        let req = DataStoreServiceRequest::WritePrivateOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::WritePrivateOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn fetch_private_input(&mut self, req: FetchPrivateInputRequest) -> mesatee_core::Result<FetchPrivateInputResponse> {
        let req = DataStoreServiceRequest::FetchPrivateInput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::FetchPrivateInput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn fetch_private_output(&mut self, req: FetchPrivateOutputRequest) -> mesatee_core::Result<FetchPrivateOutputResponse> {
        let req = DataStoreServiceRequest::FetchPrivateOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::FetchPrivateOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum FusionDataServiceRequest {
    RegisterFusionOutput(RegisterFusionOutputRequest),
    FetchFusionData(FetchFusionDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FusionDataServiceResponse {
    RegisterFusionOutput(RegisterFusionOutputResponse),
    FetchFusionData(FetchFusionDataResponse),
}
pub trait FusionDataServiceService {
    fn register_fusion_output(req: RegisterFusionOutputRequest) -> mesatee_core::Result<RegisterFusionOutputResponse>;
    fn fetch_fusion_data(req: FetchFusionDataRequest) -> mesatee_core::Result<FetchFusionDataResponse>;
    fn dispatch(&self, req: FusionDataServiceRequest) -> mesatee_core::Result<FusionDataServiceResponse> {
        match req {
            FusionDataServiceRequest::RegisterFusionOutput(req) => Self::register_fusion_output(req).map(FusionDataServiceResponse::RegisterFusionOutput),            FusionDataServiceRequest::FetchFusionData(req) => Self::fetch_fusion_data(req).map(FusionDataServiceResponse::FetchFusionData),        }
    }
}
pub struct FusionDataServiceClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<FusionDataServiceRequest, FusionDataServiceResponse>,
}

impl FusionDataServiceClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<FusionDataServiceRequest, FusionDataServiceResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(FusionDataServiceClient { channel })
    }
}
impl FusionDataServiceClient {
    pub fn register_fusion_output(&mut self, req: RegisterFusionOutputRequest) -> mesatee_core::Result<RegisterFusionOutputResponse> {
        let req = FusionDataServiceRequest::RegisterFusionOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FusionDataServiceResponse::RegisterFusionOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn fetch_fusion_data(&mut self, req: FetchFusionDataRequest) -> mesatee_core::Result<FetchFusionDataResponse> {
        let req = FusionDataServiceRequest::FetchFusionData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FusionDataServiceResponse::FetchFusionData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchFunctionRequest {
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FetchFunctionResponse {
    #[prost(message, required, tag="1")]
    pub function: tdfs_common_proto::function_common_proto::FunctionDefinition,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum FunctionStoreServiceRequest {
    FetchFunction(FetchFunctionRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FunctionStoreServiceResponse {
    FetchFunction(FetchFunctionResponse),
}
pub trait FunctionStoreServiceService {
    fn fetch_function(req: FetchFunctionRequest) -> mesatee_core::Result<FetchFunctionResponse>;
    fn dispatch(&self, req: FunctionStoreServiceRequest) -> mesatee_core::Result<FunctionStoreServiceResponse> {
        match req {
            FunctionStoreServiceRequest::FetchFunction(req) => Self::fetch_function(req).map(FunctionStoreServiceResponse::FetchFunction),        }
    }
}
pub struct FunctionStoreServiceClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<FunctionStoreServiceRequest, FunctionStoreServiceResponse>,
}

impl FunctionStoreServiceClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<FunctionStoreServiceRequest, FunctionStoreServiceResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(FunctionStoreServiceClient { channel })
    }
}
impl FunctionStoreServiceClient {
    pub fn fetch_function(&mut self, req: FetchFunctionRequest) -> mesatee_core::Result<FetchFunctionResponse> {
        let req = FunctionStoreServiceRequest::FetchFunction(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FunctionStoreServiceResponse::FetchFunction(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
