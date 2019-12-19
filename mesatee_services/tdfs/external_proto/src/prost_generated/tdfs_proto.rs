/// Request for registering a private input
/// The input includes: the logical access info (key config), physical access info (storage path)
/// and meta information to check the integrity before and after encryption
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterPrivateInputRequest {
    /// Key config for file decryption
    #[prost(message, required, tag="1")]
    pub config: kms_proto::proto::KeyConfig,
    /// Meta information
    #[prost(message, required, tag="2")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    /// Storage information: where to access the input
    #[prost(message, required, tag="3")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for registering a private input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterPrivateInputResponse {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Request for registering a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterPrivateOutputRequest {
    /// Key config for file encrytpion
    #[prost(message, required, tag="1")]
    pub config: kms_proto::proto::KeyConfig,
    /// storage information
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Request for registering a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterPrivateOutputResponse {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Request for reading the meta and status of a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadPrivateOutputRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for reading the meta and status of a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadPrivateOutputResponse {
    /// Meta information. If output is not written, the meta infomation is None. 
    #[prost(message, optional, tag="1")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// Whether the output has been written by a task. 
    #[prost(bool, required, tag="2")]
    pub is_complete: bool,
}
/// Request for deleting a private data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeletePrivateDataRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for deleting a private data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeletePrivateDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Request for reading the meta and status of a fusion data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadFusionDataRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for reading the meta and status of a fusion data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadFusionDataResponse {
    /// Meta information. If output is not written, the meta infomation is None. 
    #[prost(message, optional, tag="1")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// Owners of the data
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// Which task generates the fustion data
    #[prost(string, required, tag="3")]
    pub source_task_id: std::string::String,
    /// Whether the output has been written by a task. 
    #[prost(bool, required, tag="4")]
    pub is_complete: bool,
}
/// Request for deleting a fusion data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteFusionDataRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    /// Used for authentication
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for deleting a fusion data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteFusionDataResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum DataStoreServiceRequest {
    RegisterPrivateInput(RegisterPrivateInputRequest),
    RegisterPrivateOutput(RegisterPrivateOutputRequest),
    ReadPrivateOutput(ReadPrivateOutputRequest),
    DeletePrivateData(DeletePrivateDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DataStoreServiceResponse {
    RegisterPrivateInput(RegisterPrivateInputResponse),
    RegisterPrivateOutput(RegisterPrivateOutputResponse),
    ReadPrivateOutput(ReadPrivateOutputResponse),
    DeletePrivateData(DeletePrivateDataResponse),
}
pub trait DataStoreServiceService {
    fn register_private_input(req: RegisterPrivateInputRequest) -> mesatee_core::Result<RegisterPrivateInputResponse>;
    fn register_private_output(req: RegisterPrivateOutputRequest) -> mesatee_core::Result<RegisterPrivateOutputResponse>;
    fn read_private_output(req: ReadPrivateOutputRequest) -> mesatee_core::Result<ReadPrivateOutputResponse>;
    fn delete_private_data(req: DeletePrivateDataRequest) -> mesatee_core::Result<DeletePrivateDataResponse>;
    fn dispatch(&self, req: DataStoreServiceRequest) -> mesatee_core::Result<DataStoreServiceResponse> {
        let authenticated = match req {
            DataStoreServiceRequest::RegisterPrivateInput(ref req) => req.creds.get_creds().auth(),
            DataStoreServiceRequest::RegisterPrivateOutput(ref req) => req.creds.get_creds().auth(),
            DataStoreServiceRequest::ReadPrivateOutput(ref req) => req.creds.get_creds().auth(),
            DataStoreServiceRequest::DeletePrivateData(ref req) => req.creds.get_creds().auth(),
        };
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
        match req {
            DataStoreServiceRequest::RegisterPrivateInput(req) => Self::register_private_input(req).map(DataStoreServiceResponse::RegisterPrivateInput),            DataStoreServiceRequest::RegisterPrivateOutput(req) => Self::register_private_output(req).map(DataStoreServiceResponse::RegisterPrivateOutput),            DataStoreServiceRequest::ReadPrivateOutput(req) => Self::read_private_output(req).map(DataStoreServiceResponse::ReadPrivateOutput),            DataStoreServiceRequest::DeletePrivateData(req) => Self::delete_private_data(req).map(DataStoreServiceResponse::DeletePrivateData),        }
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
    pub fn register_private_input(&mut self, req: RegisterPrivateInputRequest) -> mesatee_core::Result<RegisterPrivateInputResponse> {
        let req = DataStoreServiceRequest::RegisterPrivateInput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::RegisterPrivateInput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn register_private_output(&mut self, req: RegisterPrivateOutputRequest) -> mesatee_core::Result<RegisterPrivateOutputResponse> {
        let req = DataStoreServiceRequest::RegisterPrivateOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::RegisterPrivateOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn read_private_output(&mut self, req: ReadPrivateOutputRequest) -> mesatee_core::Result<ReadPrivateOutputResponse> {
        let req = DataStoreServiceRequest::ReadPrivateOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::ReadPrivateOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn delete_private_data(&mut self, req: DeletePrivateDataRequest) -> mesatee_core::Result<DeletePrivateDataResponse> {
        let req = DataStoreServiceRequest::DeletePrivateData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            DataStoreServiceResponse::DeletePrivateData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum FusionDataServiceRequest {
    ReadFusionData(ReadFusionDataRequest),
    DeleteFusionData(DeleteFusionDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FusionDataServiceResponse {
    ReadFusionData(ReadFusionDataResponse),
    DeleteFusionData(DeleteFusionDataResponse),
}
pub trait FusionDataServiceService {
    fn read_fusion_data(req: ReadFusionDataRequest) -> mesatee_core::Result<ReadFusionDataResponse>;
    fn delete_fusion_data(req: DeleteFusionDataRequest) -> mesatee_core::Result<DeleteFusionDataResponse>;
    fn dispatch(&self, req: FusionDataServiceRequest) -> mesatee_core::Result<FusionDataServiceResponse> {
        let authenticated = match req {
            FusionDataServiceRequest::ReadFusionData(ref req) => req.creds.get_creds().auth(),
            FusionDataServiceRequest::DeleteFusionData(ref req) => req.creds.get_creds().auth(),
        };
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
        match req {
            FusionDataServiceRequest::ReadFusionData(req) => Self::read_fusion_data(req).map(FusionDataServiceResponse::ReadFusionData),            FusionDataServiceRequest::DeleteFusionData(req) => Self::delete_fusion_data(req).map(FusionDataServiceResponse::DeleteFusionData),        }
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
    pub fn read_fusion_data(&mut self, req: ReadFusionDataRequest) -> mesatee_core::Result<ReadFusionDataResponse> {
        let req = FusionDataServiceRequest::ReadFusionData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FusionDataServiceResponse::ReadFusionData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn delete_fusion_data(&mut self, req: DeleteFusionDataRequest) -> mesatee_core::Result<DeleteFusionDataResponse> {
        let req = FusionDataServiceRequest::DeleteFusionData(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FusionDataServiceResponse::DeleteFusionData(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
