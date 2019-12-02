#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct CreateKeyRequest {
    #[prost(enumeration = "EncType", required, tag = "1")]
    pub enc_type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AeadConfig {
    #[prost(bytes, required, tag = "1")]
    #[serde(with = "crate::base64_coder")]
    pub key: std::vec::Vec<u8>,
    #[prost(bytes, required, tag = "2")]
    #[serde(with = "crate::base64_coder")]
    pub nonce: std::vec::Vec<u8>,
    #[prost(bytes, required, tag = "3")]
    #[serde(with = "crate::base64_coder")]
    pub ad: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ProtectedFsConfig {
    #[prost(bytes, required, tag = "1")]
    #[serde(with = "crate::base64_coder")]
    pub key: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct KeyConfig {
    #[prost(oneof = "key_config::Config", tags = "1, 2")]
    pub config: ::std::option::Option<key_config::Config>,
}
pub mod key_config {
    #[derive(
        Clone, PartialEq, ::prost::Oneof, serde_derive::Serialize, serde_derive::Deserialize,
    )]
    pub enum Config {
        #[prost(message, tag = "1")]
        Aead(super::AeadConfig),
        #[prost(message, tag = "2")]
        ProtectedFs(super::ProtectedFsConfig),
    }
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct CreateKeyResponse {
    #[prost(string, required, tag = "1")]
    pub key_id: std::string::String,
    #[prost(message, required, tag = "2")]
    pub config: KeyConfig,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetKeyRequest {
    #[prost(string, required, tag = "1")]
    pub key_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetKeyResponse {
    #[prost(message, required, tag = "1")]
    pub config: KeyConfig,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteKeyRequest {
    #[prost(string, required, tag = "1")]
    pub key_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DeleteKeyResponse {
    #[prost(message, required, tag = "1")]
    pub config: KeyConfig,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum EncType {
    Aead = 0,
    ProtectedFs = 1,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum KMSRequest {
    GetKey(GetKeyRequest),
    DelKey(DeleteKeyRequest),
    CreateKey(CreateKeyRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum KMSResponse {
    GetKey(GetKeyResponse),
    DelKey(DeleteKeyResponse),
    CreateKey(CreateKeyResponse),
}
pub trait KMSService {
    fn get_key(req: GetKeyRequest) -> mesatee_core::Result<GetKeyResponse>;
    fn del_key(req: DeleteKeyRequest) -> mesatee_core::Result<DeleteKeyResponse>;
    fn create_key(req: CreateKeyRequest) -> mesatee_core::Result<CreateKeyResponse>;
    fn dispatch(&self, req: KMSRequest) -> mesatee_core::Result<KMSResponse> {
        match req {
            KMSRequest::GetKey(req) => Self::get_key(req).map(KMSResponse::GetKey),
            KMSRequest::DelKey(req) => Self::del_key(req).map(KMSResponse::DelKey),
            KMSRequest::CreateKey(req) => Self::create_key(req).map(KMSResponse::CreateKey),
        }
    }
}
pub struct KMSClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<KMSRequest, KMSResponse>,
}

impl KMSClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<KMSRequest, KMSResponse>::new(
                    addr,
                    enclave_addr,
                )?
            }
        };
        Ok(KMSClient { channel })
    }
}
impl KMSClient {
    pub fn get_key(&mut self, req: GetKeyRequest) -> mesatee_core::Result<GetKeyResponse> {
        let req = KMSRequest::GetKey(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            KMSResponse::GetKey(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn del_key(&mut self, req: DeleteKeyRequest) -> mesatee_core::Result<DeleteKeyResponse> {
        let req = KMSRequest::DelKey(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            KMSResponse::DelKey(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }

    pub fn create_key(&mut self, req: CreateKeyRequest) -> mesatee_core::Result<CreateKeyResponse> {
        let req = KMSRequest::CreateKey(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            KMSResponse::CreateKey(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(
                mesatee_core::ErrorKind::RPCResponseError,
            )),
        }
    }
}
