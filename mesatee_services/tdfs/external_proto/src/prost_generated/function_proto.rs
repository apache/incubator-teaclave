/// Request for registering a function
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterFunctionRequest {
    /// function description
    #[prost(string, required, tag="1")]
    pub description: std::string::String,
    /// script content
    #[prost(bytes, required, tag="2")]
    pub script: std::vec::Vec<u8>,
    /// whether the function is private or public. 
    /// if the function is private, the task need the owner's approval.  
    #[prost(bool, required, tag="3")]
    pub public: bool,
    /// input arguments
    #[prost(message, repeated, tag="4")]
    pub input_args: ::std::vec::Vec<tdfs_common_proto::function_common_proto::Arg>,
    /// output arguments
    #[prost(message, repeated, tag="5")]
    pub output_args: ::std::vec::Vec<tdfs_common_proto::function_common_proto::Arg>,
    /// relationship between input and output
    #[prost(message, repeated, tag="6")]
    pub arg_relationships: ::std::vec::Vec<tdfs_common_proto::function_common_proto::ArgRelationship>,
    /// user credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for registering a function
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterFunctionResponse {
    /// function identifier
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
}
/// Request for reading a function
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadFunctionRequest {
    /// function identifier
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
    /// user credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for reading a function
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ReadFunctionResponse {
    /// function definition
    /// only the owner can read the function, unless the function is public
    #[prost(message, required, tag="1")]
    pub function: tdfs_common_proto::function_common_proto::FunctionDefinition,
}
/// Request for deleting a function. Only the owner can delete it.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteFunctionRequest {
    /// function identifier
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
    /// user credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for deleting a function
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteFunctionResponse {
    /// success or not
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum FunctionStoreServiceRequest {
    RegisterFunction(RegisterFunctionRequest),
    ReadFunction(ReadFunctionRequest),
    DeleteFunction(DeleteFunctionRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum FunctionStoreServiceResponse {
    RegisterFunction(RegisterFunctionResponse),
    ReadFunction(ReadFunctionResponse),
    DeleteFunction(DeleteFunctionResponse),
}
pub trait FunctionStoreServiceService {
    fn register_function(req: RegisterFunctionRequest) -> mesatee_core::Result<RegisterFunctionResponse>;
    fn read_function(req: ReadFunctionRequest) -> mesatee_core::Result<ReadFunctionResponse>;
    fn delete_function(req: DeleteFunctionRequest) -> mesatee_core::Result<DeleteFunctionResponse>;
    fn dispatch(&self, req: FunctionStoreServiceRequest) -> mesatee_core::Result<FunctionStoreServiceResponse> {
        match req {
            FunctionStoreServiceRequest::RegisterFunction(req) => Self::register_function(req).map(FunctionStoreServiceResponse::RegisterFunction),            FunctionStoreServiceRequest::ReadFunction(req) => Self::read_function(req).map(FunctionStoreServiceResponse::ReadFunction),            FunctionStoreServiceRequest::DeleteFunction(req) => Self::delete_function(req).map(FunctionStoreServiceResponse::DeleteFunction),        }
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
    pub fn register_function(&mut self, req: RegisterFunctionRequest) -> mesatee_core::Result<RegisterFunctionResponse> {
        let req = FunctionStoreServiceRequest::RegisterFunction(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FunctionStoreServiceResponse::RegisterFunction(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn read_function(&mut self, req: ReadFunctionRequest) -> mesatee_core::Result<ReadFunctionResponse> {
        let req = FunctionStoreServiceRequest::ReadFunction(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FunctionStoreServiceResponse::ReadFunction(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn delete_function(&mut self, req: DeleteFunctionRequest) -> mesatee_core::Result<DeleteFunctionResponse> {
        let req = FunctionStoreServiceRequest::DeleteFunction(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            FunctionStoreServiceResponse::DeleteFunction(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
