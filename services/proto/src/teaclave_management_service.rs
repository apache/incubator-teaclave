use crate::teaclave_management_service_proto as proto;

pub use proto::TeaclaveManagement;
pub use proto::TeaclaveManagementClient;
pub use proto::TeaclaveManagementRequest;
pub use proto::TeaclaveManagementResponse;

pub type RegisterInputFileRequest = crate::teaclave_frontend_service::RegisterInputFileRequest;
pub type RegisterInputFileResponse = crate::teaclave_frontend_service::RegisterInputFileResponse;
pub type RegisterOutputFileRequest = crate::teaclave_frontend_service::RegisterOutputFileRequest;
pub type RegisterOutputFileResponse = crate::teaclave_frontend_service::RegisterOutputFileResponse;
pub type RegisterFusionOutputRequest =
    crate::teaclave_frontend_service::RegisterFusionOutputRequest;
pub type RegisterFusionOutputResponse =
    crate::teaclave_frontend_service::RegisterFusionOutputResponse;
pub type RegisterInputFromOutputRequest =
    crate::teaclave_frontend_service::RegisterInputFromOutputRequest;
pub type RegisterInputFromOutputResponse =
    crate::teaclave_frontend_service::RegisterInputFromOutputResponse;
pub type GetInputFileRequest = crate::teaclave_frontend_service::GetInputFileRequest;
pub type GetInputFileResponse = crate::teaclave_frontend_service::GetInputFileResponse;
pub type GetOutputFileRequest = crate::teaclave_frontend_service::GetOutputFileRequest;
pub type GetOutputFileResponse = crate::teaclave_frontend_service::GetOutputFileResponse;
pub type RegisterFunctionRequest = crate::teaclave_frontend_service::RegisterFunctionRequest;
pub type RegisterFunctionResponse = crate::teaclave_frontend_service::RegisterFunctionResponse;
pub type GetFunctionRequest = crate::teaclave_frontend_service::GetFunctionRequest;
pub type GetFunctionResponse = crate::teaclave_frontend_service::GetFunctionResponse;
pub type CreateTaskRequest = crate::teaclave_frontend_service::CreateTaskRequest;
pub type CreateTaskResponse = crate::teaclave_frontend_service::CreateTaskResponse;
pub type GetTaskRequest = crate::teaclave_frontend_service::GetTaskRequest;
pub type GetTaskResponse = crate::teaclave_frontend_service::GetTaskResponse;
pub type AssignDataRequest = crate::teaclave_frontend_service::AssignDataRequest;
pub type AssignDataResponse = crate::teaclave_frontend_service::AssignDataResponse;
pub type ApproveTaskRequest = crate::teaclave_frontend_service::ApproveTaskRequest;
pub type ApproveTaskResponse = crate::teaclave_frontend_service::ApproveTaskResponse;
pub type InvokeTaskRequest = crate::teaclave_frontend_service::InvokeTaskRequest;
pub type InvokeTaskResponse = crate::teaclave_frontend_service::InvokeTaskResponse;
