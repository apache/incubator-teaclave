#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Credential {
    #[prost(string, required, tag="1")]
    pub user_id: std::string::String,
    #[prost(string, required, tag="2")]
    pub user_token: std::string::String,
}
