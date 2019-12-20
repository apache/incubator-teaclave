/// Function information
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Function {
    /// Built-in function only needs function_id
    #[prost(string, optional, tag="1")]
    pub function_id: ::std::option::Option<std::string::String>,
    /// script content
    #[prost(bytes, required, tag="2")]
    pub script: std::vec::Vec<u8>,
}
/// Input Arugment
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InputArg {
    /// Meta information is used to verify the data integrity. 
    #[prost(message, required, tag="1")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    /// Storage information is used to access the physical data. 
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// Key config is used to decrypt the data.
    #[prost(message, required, tag="3")]
    pub config: kms_proto::proto::KeyConfig,
}
/// Output Argument
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct OutputArg {
    /// Storage information is used to access the physical data. 
    #[prost(message, required, tag="1")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    /// Key config is used to encrypt the data.
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
}
/// ExecutableTask contains all the information the worker is required to execute the Task.
/// Compared to TaskAgreement, data_id is replaced by meta information, physical access information and logical access information.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ExecutableTask {
    /// Task identifer
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    /// Task token is used for the worker to update the Task status and output meta
    #[prost(string, required, tag="2")]
    pub task_token: std::string::String,
    /// Function information
    #[prost(message, required, tag="3")]
    pub function: Function,
    /// Input Arguments
    #[prost(map="string, message", tag="4")]
    pub input_arguments: ::std::collections::HashMap<std::string::String, InputArg>,
    /// Output Arguments
    #[prost(map="string, message", tag="5")]
    pub output_argments: ::std::collections::HashMap<std::string::String, OutputArg>,
}
/// Request for updating Task status and output meta.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct UpdateTaskRequest {
    /// Task identifier
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    /// Task token 
    #[prost(string, required, tag="2")]
    pub task_token: std::string::String,
    /// Task status
    #[prost(enumeration="TaskStatus", required, tag="3")]
    pub status: i32,
    /// Output meta
    #[prost(map="string, message", tag="4")]
    pub output_meta: ::std::collections::HashMap<std::string::String, tdfs_common_proto::data_common_proto::DataMeta>,
}
/// Response for updating Task status and output meta.
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct UpdateTaskResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
/// Task status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum TaskStatus {
    /// If the Task is scheduled and being executed in a worker, the status is Running
    Running = 3,
    /// If the Task is executed successfully, the status is Successful
    Successful = 4,
    /// If the Task is failed to be executed, the status is Failed.
    Failed = 5,
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]pub enum TaskServiceRequest {
    UpdateTask(UpdateTaskRequest),
}
#[derive(Clone, serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TaskServiceResponse {
    UpdateTask(UpdateTaskResponse),
}
pub trait TaskServiceService {
    fn update_task(req: UpdateTaskRequest) -> mesatee_core::Result<UpdateTaskResponse>;
    fn dispatch(&self, req: TaskServiceRequest) -> mesatee_core::Result<TaskServiceResponse> {
        match req {
            TaskServiceRequest::UpdateTask(req) => Self::update_task(req).map(TaskServiceResponse::UpdateTask),        }
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
    pub fn update_task(&mut self, req: UpdateTaskRequest) -> mesatee_core::Result<UpdateTaskResponse> {
        let req = TaskServiceRequest::UpdateTask(req);
        let resp = self.channel.invoke(req)?;
        match resp {
            TaskServiceResponse::UpdateTask(resp) => Ok(resp),
            _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
        }
    }
}
