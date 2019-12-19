/// Argument Defintion
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Arg {
    /// argument name
    #[prost(string, required, tag="1")]
    pub name: std::string::String,
    /// argument type
    #[prost(string, required, tag="2")]
    pub arg_type: std::string::String,
    /// argument description
    #[prost(string, optional, tag="3")]
    pub description: ::std::option::Option<std::string::String>,
}
/// Argument relationship: input and output should have same owner.
/// This is a reminder for the users and later the developer may check the relationship in the script.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ArgRelationship {
    #[prost(string, required, tag="1")]
    pub input_name: std::string::String,
    #[prost(string, required, tag="2")]
    pub output_name: std::string::String,
}
/// The defintion of the function.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct FunctionDefinition {
    /// function identifier. 
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
    /// the description of the function
    #[prost(string, required, tag="2")]
    pub description: std::string::String,
    /// function owner
    #[prost(string, required, tag="3")]
    pub owner: std::string::String,
    /// script content
    #[prost(bytes, required, tag="4")]
    pub script: std::vec::Vec<u8>,
    /// hash of the script
    #[prost(string, required, tag="5")]
    pub script_hash: std::string::String,
    /// whether the function is public or private.
    /// if the function is private, the task need the owner's approval.  
    #[prost(bool, required, tag="6")]
    pub public: bool,
    /// input arguments
    #[prost(message, repeated, tag="7")]
    pub input_args: ::std::vec::Vec<Arg>,
    /// output arguments
    #[prost(message, repeated, tag="8")]
    pub output_args: ::std::vec::Vec<Arg>,
    /// relationship between input and output
    #[prost(message, repeated, tag="9")]
    pub arg_relationships: ::std::vec::Vec<ArgRelationship>,
    /// hash of the function defintion (script content and signatures are not included). 
    #[prost(string, required, tag="10")]
    pub function_hash: std::string::String,
}
