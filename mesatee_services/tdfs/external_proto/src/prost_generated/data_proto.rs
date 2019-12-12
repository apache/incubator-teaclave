/// Request for registering an input
/// The input contains: the logical access info (key config), physical access info (storage path)
/// and meta information to check the integrity before and after encryption
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterInputRequest {
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
/// Response for registering an input
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterInputResponse {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Request for registering a private output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterOutputRequest {
    /// Encryption type: AEAD, ProtectedFS
    #[prost(enumeration="kms_proto::proto::EncType", required, tag="1")]
    pub enc_type: i32,
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
pub struct RegisterOutputResponse {
    /// data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
}
/// Request for reading the output
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadOutputRequest {
    /// Data identifier
    #[prost(string, required, tag="1")]
    pub data_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for reading the output.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadOutputResponse {
    /// Meta information. If output is not written, the meta infomation is None. 
    #[prost(message, optional, tag="1")]
    pub meta: ::std::option::Option<tdfs_common_proto::data_common_proto::DataMeta>,
    /// Key config for file decryption. If output is not written, the key config won't be returned. 
    #[prost(message, optional, tag="2")]
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
    RegisterInput(RegisterInputRequest),
    RegisterOutput(RegisterOutputRequest),
    ReadOutput(ReadOutputRequest),
    DeleteData(DeleteDataRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TDFSExternalResponse {
    RegisterInput(RegisterInputResponse),
    RegisterOutput(RegisterOutputResponse),
    ReadOutput(ReadOutputResponse),
    DeleteData(DeleteDataResponse),
}
pub trait TDFSExternalService {
    fn register_input(req: RegisterInputRequest) -> mesatee_core::Result<RegisterInputResponse>;
    fn register_output(req: RegisterOutputRequest) -> mesatee_core::Result<RegisterOutputResponse>;
    fn read_output(req: ReadOutputRequest) -> mesatee_core::Result<ReadOutputResponse>;
    fn delete_data(req: DeleteDataRequest) -> mesatee_core::Result<DeleteDataResponse>;
    fn dispatch(&self, req: TDFSExternalRequest) -> mesatee_core::Result<TDFSExternalResponse> {
        let authenticated = match req {
            TDFSExternalRequest::RegisterInput(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::RegisterOutput(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::ReadOutput(ref req) => req.creds.get_creds().auth(),
            TDFSExternalRequest::DeleteData(ref req) => req.creds.get_creds().auth(),
        };
        if !authenticated {
            return Err(mesatee_core::Error::from(mesatee_core::ErrorKind::PermissionDenied));
        }
        match req {
            TDFSExternalRequest::RegisterInput(req) => Self::register_input(req).map(TDFSExternalResponse::RegisterInput),            TDFSExternalRequest::RegisterOutput(req) => Self::register_output(req).map(TDFSExternalResponse::RegisterOutput),            TDFSExternalRequest::ReadOutput(req) => Self::read_output(req).map(TDFSExternalResponse::ReadOutput),            TDFSExternalRequest::DeleteData(req) => Self::delete_data(req).map(TDFSExternalResponse::DeleteData),        }
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
    pub fn register_input(&mut self, req: RegisterInputRequest) -> mesatee_core::Result<RegisterInputResponse> {
        let req = TDFSExternalRequest::RegisterInput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::RegisterInput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn register_output(&mut self, req: RegisterOutputRequest) -> mesatee_core::Result<RegisterOutputResponse> {
        let req = TDFSExternalRequest::RegisterOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::RegisterOutput(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn read_output(&mut self, req: ReadOutputRequest) -> mesatee_core::Result<ReadOutputResponse> {
        let req = TDFSExternalRequest::ReadOutput(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TDFSExternalResponse::ReadOutput(resp) => Ok(resp),
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
