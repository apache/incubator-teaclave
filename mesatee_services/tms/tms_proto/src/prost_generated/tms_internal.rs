#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct Function {
    #[prost(bytes, required, tag="1")]
    pub script: std::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct InputArg {
    #[prost(message, required, tag="1")]
    pub meta: tdfs_common_proto::data_common_proto::DataMeta,
    #[prost(message, required, tag="2")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    #[prost(message, required, tag="3")]
    pub config: kms_proto::proto::KeyConfig,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct OutputArg {
    #[prost(message, required, tag="1")]
    pub storage_info: tdfs_common_proto::data_common_proto::DataStorageInfo,
    #[prost(message, required, tag="2")]
    pub config: kms_proto::proto::KeyConfig,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ExecutableTask {
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    #[prost(string, required, tag="2")]
    pub task_token: std::string::String,
    #[prost(message, required, tag="3")]
    pub function: Function,
    #[prost(map="string, message", tag="4")]
    pub input_arguments: ::std::collections::HashMap<std::string::String, InputArg>,
    #[prost(map="string, message", tag="5")]
    pub output_argments: ::std::collections::HashMap<std::string::String, OutputArg>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct UpdateTaskRequest {
    #[prost(string, required, tag="1")]
    pub task_id: std::string::String,
    #[prost(string, required, tag="2")]
    pub task_token: std::string::String,
    #[prost(enumeration="TaskStatus", required, tag="3")]
    pub status: i32,
    #[prost(map="string, message", tag="4")]
    pub output_meta: ::std::collections::HashMap<std::string::String, tdfs_common_proto::data_common_proto::DataMeta>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub struct UpdateTaskResponse {
    #[prost(bool, required, tag="1")]
    pub success: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum TaskStatus {
    Running = 3,
    Successful = 4,
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
