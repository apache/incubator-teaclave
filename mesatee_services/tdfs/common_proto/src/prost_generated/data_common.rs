/// Meta information of a data
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DataMeta {
    /// Hash of the encrypted file
    /// It can be use to check the integrity of the encrypted file without key
    #[prost(string, required, tag="1")]
    pub hash: std::string::String,
    /// Size of the encrypted file
    #[prost(uint64, required, tag="2")]
    pub size: u64,
    /// Hash of the original file
    /// It can be use to check the integrity of the file after file is decrypted
    #[prost(string, required, tag="3")]
    pub plaintxt_hash: std::string::String,
    /// Size of the original file
    #[prost(uint64, required, tag="4")]
    pub plaintext_size: u64,
}
/// Where to access the data
/// The uri can contains the scheme, path, token and so on.
/// For example, internal://user_id/file1 or s3://bucket_id/path?access_id=xxx&access_token=xxx
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct DataStorageInfo {
    #[prost(string, required, tag="1")]
    pub uri: std::string::String,
}
