#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DataMeta {
    #[prost(string, required, tag="1")]
    pub hash: std::string::String,
    #[prost(uint64, required, tag="2")]
    pub size: u64,
    #[prost(string, required, tag="3")]
    pub plaintxt_hash: std::string::String,
    #[prost(uint64, required, tag="4")]
    pub plaintext_size: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DataStorageInfo {
    #[prost(string, required, tag="1")]
    pub uri: std::string::String,
}
