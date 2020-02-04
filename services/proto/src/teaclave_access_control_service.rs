use crate::teaclave_access_control_service_proto as proto;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::prelude::v1::*;

pub use proto::TeaclaveAccessControl;
pub use proto::TeaclaveAccessControlClient;
pub use proto::TeaclaveAccessControlRequest;
pub use proto::TeaclaveAccessControlResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeDataRequest {
    pub subject_user_id: String,
    pub object_data_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeDataResponse {
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeFunctionRequest {
    pub subject_user_id: String,
    pub object_function_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeFunctionResponse {
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeTaskRequest {
    pub subject_user_id: String,
    pub object_task_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeTaskResponse {
    pub accept: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeStagedTaskRequest {
    pub subject_task_id: String,
    pub object_function_id: String,
    pub object_input_data_id_list: Vec<String>,
    pub object_output_data_id_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorizeStagedTaskResponse {
    pub accept: bool,
}

impl std::convert::TryFrom<proto::AuthorizeDataRequest> for AuthorizeDataRequest {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeDataRequest) -> Result<Self> {
        let ret = Self {
            subject_user_id: proto.subject_user_id,
            object_data_id: proto.object_data_id,
        };

        Ok(ret)
    }
}

impl From<AuthorizeDataRequest> for proto::AuthorizeDataRequest {
    fn from(request: AuthorizeDataRequest) -> Self {
        Self {
            subject_user_id: request.subject_user_id,
            object_data_id: request.object_data_id,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeDataResponse> for AuthorizeDataResponse {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeDataResponse) -> Result<Self> {
        Ok(Self {
            accept: proto.accept,
        })
    }
}

impl From<AuthorizeDataResponse> for proto::AuthorizeDataResponse {
    fn from(response: AuthorizeDataResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeFunctionRequest> for AuthorizeFunctionRequest {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeFunctionRequest) -> Result<Self> {
        let ret = Self {
            subject_user_id: proto.subject_user_id,
            object_function_id: proto.object_function_id,
        };

        Ok(ret)
    }
}

impl From<AuthorizeFunctionRequest> for proto::AuthorizeFunctionRequest {
    fn from(request: AuthorizeFunctionRequest) -> Self {
        Self {
            subject_user_id: request.subject_user_id,
            object_function_id: request.object_function_id,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeFunctionResponse> for AuthorizeFunctionResponse {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeFunctionResponse) -> Result<Self> {
        Ok(Self {
            accept: proto.accept,
        })
    }
}

impl From<AuthorizeFunctionResponse> for proto::AuthorizeFunctionResponse {
    fn from(response: AuthorizeFunctionResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeTaskRequest> for AuthorizeTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeTaskRequest) -> Result<Self> {
        let ret = Self {
            subject_user_id: proto.subject_user_id,
            object_task_id: proto.object_task_id,
        };

        Ok(ret)
    }
}

impl From<AuthorizeTaskRequest> for proto::AuthorizeTaskRequest {
    fn from(request: AuthorizeTaskRequest) -> Self {
        Self {
            subject_user_id: request.subject_user_id,
            object_task_id: request.object_task_id,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeTaskResponse> for AuthorizeTaskResponse {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeTaskResponse) -> Result<Self> {
        Ok(Self {
            accept: proto.accept,
        })
    }
}

impl From<AuthorizeTaskResponse> for proto::AuthorizeTaskResponse {
    fn from(response: AuthorizeTaskResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeStagedTaskRequest> for AuthorizeStagedTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeStagedTaskRequest) -> Result<Self> {
        let ret = Self {
            subject_task_id: proto.subject_task_id,
            object_function_id: proto.object_function_id,
            object_input_data_id_list: proto.object_input_data_id_list,
            object_output_data_id_list: proto.object_output_data_id_list,
        };

        Ok(ret)
    }
}

impl From<AuthorizeStagedTaskRequest> for proto::AuthorizeStagedTaskRequest {
    fn from(request: AuthorizeStagedTaskRequest) -> Self {
        Self {
            subject_task_id: request.subject_task_id,
            object_function_id: request.object_function_id,
            object_input_data_id_list: request.object_input_data_id_list,
            object_output_data_id_list: request.object_output_data_id_list,
        }
    }
}

impl std::convert::TryFrom<proto::AuthorizeStagedTaskResponse> for AuthorizeStagedTaskResponse {
    type Error = Error;

    fn try_from(proto: proto::AuthorizeStagedTaskResponse) -> Result<Self> {
        Ok(Self {
            accept: proto.accept,
        })
    }
}

impl From<AuthorizeStagedTaskResponse> for proto::AuthorizeStagedTaskResponse {
    fn from(response: AuthorizeStagedTaskResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}
