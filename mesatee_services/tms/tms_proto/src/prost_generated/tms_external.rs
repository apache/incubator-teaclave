/// input argument in TaskAgreement
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InputArg {
    /// private input or fusion input
    #[prost(bool, required, tag="1")]
    pub is_private: bool,
    /// owners of the input: private - single owner, fusion - multiple owners
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// data id of the input
    #[prost(string, required, tag="3")]
    pub data_id: std::string::String,
}
/// output argument in TaskAgreement
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct OutputArg {
    /// private output or fusion output
    #[prost(bool, required, tag="1")]
    pub is_private: bool,
    /// owners of the input: private - single owner, fusion - multiple owners
    #[prost(string, repeated, tag="2")]
    pub owners: ::std::vec::Vec<std::string::String>,
    /// data id of the output
    /// fusion output may not have a data id before the TaskAgreement is accepted by the TMS
    #[prost(string, optional, tag="3")]
    pub data_id: ::std::option::Option<std::string::String>,
}
/// TaskAgreement contains all the information to invoke a task
/// But sensitive data is represented by data id.
/// Multiple participants can consent to the TaskAgreement offline.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct TaskAgreement {
    /// Task identifier
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    /// Function identifier
    #[prost(string, required, tag="2")]
    pub function_id: std::string::String,
    /// Input arguments
    #[prost(map="string, message", tag="3")]
    pub input_arguments: ::std::collections::HashMap<std::string::String, InputArg>,
    /// Output arguments
    #[prost(map="string, message", tag="4")]
    pub output_argments: ::std::collections::HashMap<std::string::String, OutputArg>,
    /// Participants
    #[prost(string, repeated, tag="5")]
    pub participants: ::std::vec::Vec<std::string::String>,
    /// Signatures of the participants
    #[prost(map="string, bytes", tag="6")]
    pub signatures: ::std::collections::HashMap<std::string::String, std::vec::Vec<u8>>,
}
/// Request for registering a Task, in order to propose a TaskAgreement
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterTaskRequest {
    /// Function identifier
    #[prost(string, required, tag="1")]
    pub function_id: std::string::String,
    /// Participants
    #[prost(string, repeated, tag="2")]
    pub participants: ::std::vec::Vec<std::string::String>,
    /// User credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for registering a Task
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct RegisterTaskResponse {
    /// initial TaskAgreement
    #[prost(message, required, tag="1")]
    pub initial_agreement: TaskAgreement,
}
/// Request for getting a Task
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetTaskRequest {
    /// Task identifier
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    /// User credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for getting a Task
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GetTaskResponse {
    /// TaskAgreement
    #[prost(message, required, tag="1")]
    pub agreement: TaskAgreement,
    /// Task Status
    #[prost(enumeration="TaskStatus", required, tag="2")]
    pub statuts: i32,
}
/// Request for invoking a Task
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InvokeTaskRequest {
    /// A complete TaskAgreement: required inputs and outputs are provided, all the participants approved the task.
    #[prost(message, required, tag="1")]
    pub complete_agreement: TaskAgreement,
    /// User credential
    #[prost(message, required, tag="99")]
    pub creds: authentication_proto::proto::Credential,
}
/// Response for invoking a Task
/// The invocation is asynchronous. Participants need to use GetTaskRequest to query the task status.
/// Participants need to use the (logical and physical) data information they provided to access the output data.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InvokeTaskResponse {
    /// whether the task is successfully invoked.
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Task status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum TaskStatus {
    /// If a Task is registered, the status is Initial
    Initial = 1,
    /// If the TaskAgreement is approved by all the participants and accepted by the TMS, the status is Pending
    Pending = 2,
    /// If the Task is scheduled and being executed in a worker, the status is Running
    Running = 3,
    /// If the Task is executed successfully, the status is Successful
    Successful = 4,
    /// If the Task is failed to be executed or accepted by the TMS, the status is Failed.
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
