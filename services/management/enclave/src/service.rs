use crate::file::{InputFile, OutputFile};
use crate::function::Function;
use crate::fusion_data::FusionData;
use crate::task::{InputData, OutputData, StagedTask, Task};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CreateTaskRequest, CreateTaskResponse, GetFunctionRequest, GetFunctionResponse,
    GetFusionDataRequest, GetFusionDataResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse,
    RegisterFunctionRequest, RegisterFunctionResponse, RegisterInputFileRequest,
    RegisterInputFileResponse, RegisterOutputFileRequest, RegisterOutputFileResponse, TaskStatus,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagement;
use teaclave_proto::teaclave_storage_service::{
    EnqueueRequest, GetRequest, PutRequest, TeaclaveStorageClient,
};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclaveManagementError {
    #[error("invalid request")]
    InvalidRequest,
    #[error("data error")]
    DataError,
    #[error("storage error")]
    StorageError,
    #[error("permission denied")]
    PermissionDenied,
    #[error("bad task")]
    BadTask,
}

impl From<TeaclaveManagementError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveManagementError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(
    teaclave_management_service,
    TeaclaveManagement,
    TeaclaveManagementError
)]
#[derive(Clone)]
pub(crate) struct TeaclaveManagementService {
    storage_client: Arc<Mutex<TeaclaveStorageClient>>,
}

impl TeaclaveManagement for TeaclaveManagementService {
    // access control: none
    fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let input_file = InputFile::new(request.url, request.hash, request.crypto_info, user_id);
        let key = input_file.get_key_vec();
        let value = input_file
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;

        self.write_to_storage(&key, &value)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterInputFileResponse {
            data_id: input_file.data_id,
        };
        Ok(response)
    }

    // access control: none
    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let output_file = OutputFile::new(request.url, request.crypto_info, user_id);
        let key = output_file.get_key_vec();
        let value = output_file
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;

        self.write_to_storage(&key, &value)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterOutputFileResponse {
            data_id: output_file.data_id,
        };
        Ok(response)
    }

    // access control: output_file.owner == user_id
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let output_file = self.get_output_file_from_storage(&request.message.data_id)?;
        if output_file.owner != user_id {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetOutputFileResponse {
            hash: output_file.hash.unwrap_or_else(|| "".to_string()),
        };
        Ok(response)
    }

    // access control: fusion_data.data_owner_id_list.contains(user_id)
    fn get_fusion_data(
        &self,
        request: Request<GetFusionDataRequest>,
    ) -> TeaclaveServiceResponseResult<GetFusionDataResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let fusion_data = self.get_fusion_data_from_storage(&request.message.data_id)?;
        if !fusion_data.data_owner_id_list.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetFusionDataResponse {
            hash: fusion_data.hash.unwrap_or_else(|| "".to_string()),
            data_owner_id_list: fusion_data.data_owner_id_list,
        };
        Ok(response)
    }

    // access_control: none
    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let function = Function::new_from_register_request(request, user_id);
        let key = function.get_key_vec();
        let value = function
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;

        self.write_to_storage(&key, &value)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterFunctionResponse {
            function_id: function.function_id,
        };
        Ok(response)
    }

    // access control: function.is_public || function.owner == user_id
    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let function = self.get_function_from_storage(&request.message.function_id)?;
        if !(function.is_public || function.owner == user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetFunctionResponse {
            name: function.name,
            description: function.description,
            owner: function.owner,
            payload: function.payload,
            is_public: function.is_public,
            arg_list: function.arg_list,
            input_list: function.input_list,
            output_list: function.output_list,
        };
        Ok(response)
    }

    // access control: none
    // when a task is created, following rules will be verified:
    // 1) arugments match function definition
    // 2) input match function definition
    // 3) output match function definition
    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        let function = self.get_function_from_storage(&request.function_id)?;
        let mut task = Task::new(
            function,
            user_id,
            request.arg_list,
            request.input_data_owner_list,
            request.output_data_owner_list,
        )
        .map_err(|_| TeaclaveManagementError::BadTask)?;
        // register fusion data
        for (output_name, data_owner_id_list) in task.output_data_owner_list.iter() {
            if data_owner_id_list.user_id_list.len() > 1 {
                let user_id_list: Vec<String> =
                    data_owner_id_list.user_id_list.iter().cloned().collect();
                let fusion_data = self.alloc_fusion_data(user_id_list)?;
                task.output_map
                    .insert(output_name.to_string(), fusion_data.data_id);
            }
        }
        self.insert_or_update_task_to_storage(&task)?;
        Ok(CreateTaskResponse {
            task_id: task.task_id,
        })
    }

    // access control: task.participants.contains(&user_id)
    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let task: Task = self.get_task_from_storage(&request.message.task_id)?;
        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetTaskResponse {
            task_id: task.task_id,
            creator: task.creator,
            function_id: task.function_id,
            function_owner: task.function_owner,
            arg_list: task.arg_list,
            input_data_owner_list: task.input_data_owner_list,
            output_data_owner_list: task.output_data_owner_list,
            participants: task.participants,
            approved_user_list: task.approved_user_list,
            input_map: task.input_map,
            output_map: task.output_map,
            status: task.status,
        };
        Ok(response)
    }

    // access control:
    // 1) task.participants.contains(user_id)
    // 2) task.status == Created
    // 3) user can use the data:
    //    * input file: user_id == input_file.owner
    //    * output file: user_id == output_file.owner && output_file.hash.is_none()
    //    * fusion_data: fusion_data.owner_id_list.contains(user_id)
    // 4) the data can be assgined to the task:
    //    * input_data_owner_list or output_data_owner_list contains the data name
    //    * input file: DataOwnerList has one user and user == user_id
    //    * output file: DataOwnerList has one user and user == user_id
    //    * fusion data: not output and DataOwnerList == fusion_data.owner_id_list
    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        let mut task: Task = self.get_task_from_storage(&request.task_id)?;
        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Created => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        for (data_name, data_id) in request.input_map.iter() {
            if InputFile::is_input_file_id(data_id) {
                let input_file = self.get_input_file_from_storage(data_id)?;
                task.assign_input_file(data_name, &input_file, &user_id)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
            } else if FusionData::is_fusion_data_id(data_id) {
                let fusion_data = self.get_fusion_data_from_storage(data_id)?;
                task.assign_fusion_data(data_name, &fusion_data, &user_id)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            }
        }
        for (data_name, data_id) in request.output_map.iter() {
            if OutputFile::is_output_file_id(data_id) {
                let output_file = self.get_output_file_from_storage(data_id)?;
                task.assign_output_file(data_name, &output_file, &user_id)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            }
        }
        task.try_update_to_ready_status();
        self.insert_or_update_task_to_storage(&task)?;
        Ok(AssignDataResponse)
    }

    // access_control:
    // 1) task status == Ready
    // 2) user_id in task.participants
    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        let mut task: Task = self.get_task_from_storage(&request.task_id)?;
        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Ready => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        task.approved_user_list.insert(user_id);
        task.try_update_to_approved_status();
        self.insert_or_update_task_to_storage(&task)?;
        Ok(ApproveTaskResponse)
    }

    // access_control:
    // 1) task status == Approved
    // 2) user_id == task.creator
    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        let mut task: Task = self.get_task_from_storage(&request.task_id)?;
        if task.creator != user_id {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Approved => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        let function = self.get_function_from_storage(&task.function_id)?;
        let arg_list: HashMap<String, String> = task.arg_list.clone();
        let mut input_map = HashMap::new();
        let mut output_map = HashMap::new();
        for (data_name, data_id) in task.input_map.iter() {
            let input_data = if InputFile::is_input_file_id(data_id) {
                let input_file = self.get_input_file_from_storage(data_id)?;
                InputData::from_input_file(input_file)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?
            } else if FusionData::is_fusion_data_id(data_id) {
                let fusion_data = self.get_fusion_data_from_storage(data_id)?;
                InputData::from_fusion_data(fusion_data)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            };
            input_map.insert(data_name.to_string(), input_data);
        }

        for (data_name, data_id) in task.output_map.iter() {
            let output_data = if OutputFile::is_output_file_id(data_id) {
                let output_file = self.get_output_file_from_storage(data_id)?;
                OutputData::from_output_file(output_file)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?
            } else if FusionData::is_fusion_data_id(data_id) {
                let fusion_data = self.get_fusion_data_from_storage(data_id)?;
                OutputData::from_fusion_data(fusion_data)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            };
            output_map.insert(data_name.to_string(), output_data);
        }
        let staged_task = StagedTask::new(&task.task_id, function, arg_list, input_map, output_map);

        self.enqueue_staged_task(&staged_task)?;
        task.status = TaskStatus::Running;
        self.insert_or_update_task_to_storage(&task)?;
        Ok(InvokeTaskResponse)
    }
}

impl TeaclaveManagementService {
    #[cfg(test_mode)]
    fn add_mock_data(&self) -> Result<()> {
        use teaclave_proto::teaclave_frontend_service::{FunctionInput, FunctionOutput};
        use teaclave_types::TeaclaveFileCryptoInfo;
        use url::Url;
        let mut fusion_data =
            FusionData::new(vec!["mock_user2".to_string(), "mock_user3".to_string()])?;
        fusion_data.data_id = "fusion-data-mock-data".to_string();
        fusion_data.hash = Some("deadbeef".to_string());
        let key = fusion_data.get_key_vec();
        let value = fusion_data.to_vec()?;
        self.write_to_storage(&key, &value)?;

        let mut fusion_data =
            FusionData::new(vec!["mock_user1".to_string(), "mock_user2".to_string()])?;
        fusion_data.data_id = "fusion-data-mock-data2".to_string();
        let key = fusion_data.get_key_vec();
        let value = fusion_data.to_vec()?;
        self.write_to_storage(&key, &value)?;

        let function_input = FunctionInput {
            name: "input".to_string(),
            description: "input_desc".to_string(),
        };
        let function_output = FunctionOutput {
            name: "output".to_string(),
            description: "output_desc".to_string(),
        };
        let function_input2 = FunctionInput {
            name: "input2".to_string(),
            description: "input_desc".to_string(),
        };
        let function_output2 = FunctionOutput {
            name: "output2".to_string(),
            description: "output_desc".to_string(),
        };

        let native_function = Function {
            function_id: "native-mock-native-func".to_string(),
            name: "mock-native-func".to_string(),
            description: "mock-desc".to_string(),
            payload: b"mock-payload".to_vec(),
            is_public: true,
            arg_list: vec!["arg1".to_string(), "arg2".to_string()],
            input_list: vec![function_input, function_input2],
            output_list: vec![function_output, function_output2],
            owner: "teaclave".to_string(),
            is_native: true,
        };
        let key = native_function.get_key_vec();
        let value = native_function.to_vec()?;
        self.write_to_storage(&key, &value)?;

        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let user_id = "mock_user1".to_string();
        let crypto_info = TeaclaveFileCryptoInfo::default();
        let mut output_file = OutputFile::new(url, crypto_info, user_id);
        output_file.hash = Some("deadbeef".to_string());
        let key = b"output-file-mock-with-hash";
        let value = output_file.to_vec()?;
        self.write_to_storage(key, &value)?;
        Ok(())
    }

    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let channel = storage_service_endpoint.connect()?;
        let client = TeaclaveStorageClient::new(channel)?;
        let service = Self {
            storage_client: Arc::new(Mutex::new(client)),
        };
        #[cfg(test_mode)]
        service.add_mock_data()?;
        Ok(service)
    }

    fn write_to_storage(&self, key: &[u8], value: &[u8]) -> Result<()> {
        let put_request = PutRequest::new(key, value);
        let _put_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .put(put_request)?;
        Ok(())
    }

    fn read_from_storage(&self, key: &[u8]) -> Result<Vec<u8>> {
        let get_request = GetRequest::new(key);
        let get_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .get(get_request)?;
        Ok(get_response.value)
    }

    // avoid accessing other kinds of data: 1) check function_id 2) deserialization
    fn get_function_from_storage(
        &self,
        function_id: &str,
    ) -> TeaclaveServiceResponseResult<Function> {
        if !Function::is_function_id(function_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let function_key = function_id.as_bytes();
        let function_bytes = self
            .read_from_storage(function_key)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Function::from_slice(&function_bytes).map_err(|_| TeaclaveManagementError::DataError.into())
    }

    fn alloc_fusion_data(
        &self,
        user_id_list: Vec<String>,
    ) -> TeaclaveServiceResponseResult<FusionData> {
        let fusion_data =
            FusionData::new(user_id_list).map_err(|_| TeaclaveManagementError::DataError)?;
        let data_key = fusion_data.get_key_vec();
        let value = fusion_data
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;
        self.write_to_storage(&data_key, &value)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(fusion_data)
    }

    // avoid accessing other kinds of data: 1) check task_id 2) deserialization
    fn get_task_from_storage(&self, task_id: &str) -> TeaclaveServiceResponseResult<Task> {
        if !Task::is_task_id(task_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let task_key = task_id.as_bytes();
        let task_bytes = self
            .read_from_storage(task_key)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Task::from_slice(&task_bytes).map_err(|_| TeaclaveManagementError::DataError.into())
    }

    // avoid accessing other kinds of data: 1) check file_id 2) deserialization
    fn get_input_file_from_storage(
        &self,
        input_file_id: &str,
    ) -> TeaclaveServiceResponseResult<InputFile> {
        if !InputFile::is_input_file_id(input_file_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let file_key = input_file_id.as_bytes();
        let file_bytes = self
            .read_from_storage(file_key)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        InputFile::from_slice(&file_bytes).map_err(|_| TeaclaveManagementError::DataError.into())
    }

    // avoid accessing other kinds of data: 1) check fusion_data_id 2) deserialization
    fn get_fusion_data_from_storage(
        &self,
        fusion_data_id: &str,
    ) -> TeaclaveServiceResponseResult<FusionData> {
        if !FusionData::is_fusion_data_id(&fusion_data_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let key = fusion_data_id.as_bytes();
        let value = self
            .read_from_storage(key)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        FusionData::from_slice(&value).map_err(|_| TeaclaveManagementError::DataError.into())
    }

    fn get_output_file_from_storage(
        &self,
        output_file_id: &str,
    ) -> TeaclaveServiceResponseResult<OutputFile> {
        if !OutputFile::is_output_file_id(&output_file_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let key: &[u8] = output_file_id.as_bytes();
        let value = self
            .read_from_storage(key)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        OutputFile::from_slice(&value).map_err(|_| TeaclaveManagementError::DataError.into())
    }

    fn insert_or_update_task_to_storage(&self, task: &Task) -> TeaclaveServiceResponseResult<()> {
        let key = task.get_key_vec();
        let value = task
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;
        self.write_to_storage(&key, &value)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(())
    }

    fn enqueue_staged_task(&self, staged_task: &StagedTask) -> TeaclaveServiceResponseResult<()> {
        let key = StagedTask::get_queue_key();
        let value = staged_task
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;
        let enqueue_request = EnqueueRequest::new(key, value);
        let _enqueue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| TeaclaveManagementError::StorageError)?
            .enqueue(enqueue_request)?;
        Ok(())
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::collections::HashMap;
    use teaclave_proto::teaclave_frontend_service::{FunctionInput, FunctionOutput};
    use teaclave_types::{TeaclaveFileCryptoInfo, TeaclaveFileRootKey128};
    use url::Url;

    pub fn handle_input_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let hash = "a6d604b5987b693a19d94704532b5d928c2729f24dfd40745f8d03ac9ac75a8b".to_string();
        let user_id = "mock_user".to_string();
        let crypto_info = TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(
            TeaclaveFileRootKey128::new(&[0; 16]).unwrap(),
        );
        let input_file = InputFile::new(url, hash, crypto_info, user_id);
        let key = input_file.get_key_vec();
        let key_str = std::str::from_utf8(&key).unwrap();
        info!("key: {}", key_str);
        assert!(InputFile::is_input_file_id(key_str));
        let value = input_file.to_vec().unwrap();
        let deserialized_file = InputFile::from_slice(&value).unwrap();
        info!("file: {:?}", deserialized_file);
    }

    pub fn handle_output_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let user_id = "mock_user".to_string();
        let crypto_info = TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(
            TeaclaveFileRootKey128::new(&[0; 16]).unwrap(),
        );
        let output_file = OutputFile::new(url, crypto_info, user_id);
        let key = output_file.get_key_vec();
        let key_str = std::str::from_utf8(&key).unwrap();
        info!("key: {}", key_str);
        assert!(OutputFile::is_output_file_id(key_str));
        let value = output_file.to_vec().unwrap();
        let deserialized_file = OutputFile::from_slice(&value).unwrap();
        info!("file: {:?}", deserialized_file);
    }

    pub fn handle_fusion_data() {
        let fusion_data =
            FusionData::new(vec!["mock_user_a".to_string(), "mock_user_b".to_string()]).unwrap();
        let key = fusion_data.get_key_vec();
        let key_str = std::str::from_utf8(&key).unwrap();
        info!("key: {}", key_str);
        assert!(FusionData::is_fusion_data_id(key_str));
        let value = fusion_data.to_vec().unwrap();
        let deserialized_data = FusionData::from_slice(&value).unwrap();
        info!("data: {:?}", deserialized_data);
    }

    pub fn handle_function() {
        let function_input = FunctionInput {
            name: "input".to_string(),
            description: "input_desc".to_string(),
        };
        let function_output = FunctionOutput {
            name: "output".to_string(),
            description: "output_desc".to_string(),
        };
        let register_request = RegisterFunctionRequest {
            name: "mock_function".to_string(),
            description: "mock function".to_string(),
            payload: b"python script".to_vec(),
            is_public: true,
            arg_list: vec!["arg".to_string()],
            input_list: vec![function_input],
            output_list: vec![function_output],
        };
        let function =
            Function::new_from_register_request(register_request, "mock_user".to_string());
        let key = function.get_key_vec();
        let key_str = std::str::from_utf8(&key).unwrap();
        info!("key: {}", key_str);
        assert!(Function::is_function_id(key_str));
        let value = function.to_vec().unwrap();
        let deserialized_data = Function::from_slice(&value).unwrap();
        info!("data: {:?}", deserialized_data);
    }

    pub fn handle_task() {
        let function_request = RegisterFunctionRequest {
            name: "mock_function".to_string(),
            description: "mock function".to_string(),
            payload: b"python script".to_vec(),
            is_public: false,
            arg_list: vec!["arg".to_string()],
            input_list: vec![],
            output_list: vec![],
        };
        let function =
            Function::new_from_register_request(function_request, "mock_user".to_string());
        let mut arg_list = HashMap::new();
        arg_list.insert("arg".to_string(), "data".to_string());

        let task = Task::new(
            function,
            "mock_user".to_string(),
            arg_list,
            HashMap::new(),
            HashMap::new(),
        )
        .unwrap();

        let key = task.get_key_vec();
        let key_str = std::str::from_utf8(&key).unwrap();
        info!("key: {}", key_str);
        assert!(Task::is_task_id(key_str));
        let value = task.to_vec().unwrap();
        let deserialized_data = Task::from_slice(&value).unwrap();
        info!("data: {:?}", deserialized_data);
    }

    pub fn handle_staged_task() {
        let function = Function {
            function_id: "function-mock".to_string(),
            name: "mock".to_string(),
            description: "".to_string(),
            payload: b"python script".to_vec(),
            is_public: false,
            arg_list: vec![],
            input_list: vec![],
            output_list: vec![],
            owner: "mock_user".to_string(),
            is_native: true,
        };
        let mut arg_list = HashMap::new();
        arg_list.insert("arg".to_string(), "data".to_string());

        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let hash = "a6d604b5987b693a19d94704532b5d928c2729f24dfd40745f8d03ac9ac75a8b".to_string();
        let crypto_info = TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(
            TeaclaveFileRootKey128::new(&[0; 16]).unwrap(),
        );
        let input_data = InputData {
            url: url.clone(),
            hash,
            crypto_info: crypto_info.clone(),
        };
        let output_data = OutputData { url, crypto_info };
        let mut input_map = HashMap::new();
        input_map.insert("input".to_string(), input_data);
        let mut output_map = HashMap::new();
        output_map.insert("output".to_string(), output_data);

        let staged_task = StagedTask::new("task-mock", function, arg_list, input_map, output_map);

        let value = staged_task.to_vec().unwrap();
        let deserialized_data = StagedTask::from_slice(&value).unwrap();
        info!("staged task: {:?}", deserialized_data);
    }
}
