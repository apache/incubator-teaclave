#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InputArg {
    #[prost(bool, required, tag="1")]
    pub is_private: bool,
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    #[prost(string, required, tag="3")]
    pub data_id: std::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct OutputArg {
    #[prost(bool, required, tag="1")]
    pub is_private: bool,
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    #[prost(string, optional, tag="3")]
    pub data_id: ::std::option::Option<std::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct TaskAgreement {
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    #[prost(string, required, tag="2")]
    pub function_id: std::string::String,
    #[prost(map="string, message", tag="3")]
    pub input_arguments: ::std::collections::HashMap<std::string::String, InputArg>,
    #[prost(map="string, message", tag="4")]
    pub output_argments: ::std::collections::HashMap<std::string::String, OutputArg>,
    #[prost(string, repeated, tag="5")]
    pub participants: ::std::vec::Vec<std::string::String>,
    #[prost(map="string, bytes", tag="6")]
    pub signatures: ::std::collections::HashMap<std::string::String, std::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterTaskRequest {
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
    #[prost(string, repeated, tag="2")]
    pub participants: ::std::vec::Vec<std::string::String>,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterTaskResponse {
    #[prost(message, required, tag="1")]
    pub initial_agreement: TaskAgreement,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetTaskRequest {
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetTaskResponse {
    #[prost(message, required, tag="1")]
    pub agreement: TaskAgreement,
    #[prost(enumeration="TaskStatus", required, tag="2")]
    pub statuts: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InvokeTaskRequest {
    #[prost(message, required, tag="1")]
    pub complete_agreement: TaskAgreement,
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InvokeTaskResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum TaskStatus {
    Initial = 1,
    Pending = 2,
    Running = 3,
    Successful = 4,
    Failed = 5,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TaskServiceRequest {
    RegisterTask(RegisterTaskRequest),
    GetTask(GetTaskRequest),
    InvokeTask(InvokeTaskRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskServiceResponse {
    RegisterTask(RegisterTaskResponse),
    GetTask(GetTaskResponse),
    InvokeTask(InvokeTaskResponse),
}
pub trait TaskServiceService {
    fn register_task(req: RegisterTaskRequest) -> mesatee_core::Result<RegisterTaskResponse>;
    fn get_task(req: GetTaskRequest) -> mesatee_core::Result<GetTaskResponse>;
    fn invoke_task(req: InvokeTaskRequest) -> mesatee_core::Result<InvokeTaskResponse>;
    fn dispatch(&self, req: TaskServiceRequest) -> mesatee_core::Result<TaskServiceResponse> {
        match req {
            TaskServiceRequest::RegisterTask(req) => Self::register_task(req).map(TaskServiceResponse::RegisterTask),            TaskServiceRequest::GetTask(req) => Self::get_task(req).map(TaskServiceResponse::GetTask),            TaskServiceRequest::InvokeTask(req) => Self::invoke_task(req).map(TaskServiceResponse::InvokeTask),        }
    }
}
pub struct TaskServiceClient {
    channel: mesatee_core::rpc::channel::SgxTrustedChannel<TaskServiceRequest, TaskServiceResponse>,
}

impl TaskServiceClient {
    pub fn new(target: mesatee_core::config::TargetDesc) -> mesatee_core::Result<Self> {
        let addr = target.addr;
        let channel = match target.desc {
            mesatee_core::config::OutboundDesc::Sgx(enclave_addr) => {
                mesatee_core::rpc::channel::SgxTrustedChannel::<TaskServiceRequest, TaskServiceResponse>::new(addr, enclave_addr)?
            }
        };
        Ok(TaskServiceClient { channel })
    }
}
impl TaskServiceClient {
    pub fn register_task(&mut self, req: RegisterTaskRequest) -> mesatee_core::Result<RegisterTaskResponse> {
        let req = TaskServiceRequest::RegisterTask(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskServiceResponse::RegisterTask(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn get_task(&mut self, req: GetTaskRequest) -> mesatee_core::Result<GetTaskResponse> {
        let req = TaskServiceRequest::GetTask(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskServiceResponse::GetTask(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }

    pub fn invoke_task(&mut self, req: InvokeTaskRequest) -> mesatee_core::Result<InvokeTaskResponse> {
        let req = TaskServiceRequest::InvokeTask(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskServiceResponse::InvokeTask(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
