use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;
use url::Url;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(
        test_register_input_file,
        test_register_output_file,
        test_get_output_file,
        test_get_fusion_data,
        test_register_function,
        test_get_function,
        test_create_task,
        test_get_task,
        test_assign_data,
        test_approve_task,
        test_invoke_task,
    )
}

fn get_credential() -> UserCredential {
    // register user and login
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_authentication_service")
        .expect("authentication");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );
    let channel = Endpoint::new("localhost:7776")
        .config(config)
        .connect()
        .unwrap();
    let mut api_client = TeaclaveAuthenticationApiClient::new(channel).unwrap();

    let request = UserRegisterRequest::new("frontend_user", "test_password");
    let _response_result = api_client.user_register(request);

    let request = UserLoginRequest::new("frontend_user", "test_password");
    let response_result = api_client.user_login(request);
    assert!(response_result.is_ok());
    UserCredential::new("frontend_user", response_result.unwrap().token)
}

fn get_client() -> TeaclaveFrontendClient {
    let user_credential = get_credential();
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let port = &runtime_config.api_endpoints.frontend.listen_address.port();
    let channel = Endpoint::new(&format!("localhost:{}", port))
        .connect()
        .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), user_credential.id);
    metadata.insert("token".to_string(), user_credential.token);

    TeaclaveFrontendClient::new_with_metadata(channel, metadata).unwrap()
}

fn test_register_input_file() {
    let mut client = get_client();

    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client.register_input_file(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().data_id.is_empty());

    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_input_file(request);
    assert_eq!(
        response,
        Err(TeaclaveServiceResponseError::RequestError(
            "authentication error".to_string()
        ))
    );
}

fn test_register_output_file() {
    let mut client = get_client();

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client.register_output_file(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().data_id.is_empty());

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_output_file(request);
    assert!(response.is_err());
}

fn test_get_output_file() {
    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };

    let mut client = get_client();
    let response = client.register_output_file(request);
    let data_id = response.unwrap().data_id;

    let request = GetOutputFileRequest {
        data_id: data_id.clone(),
    };
    let response = client.get_output_file(request);
    assert!(response.is_ok());
    assert!(response.unwrap().hash.is_empty());

    let request = GetOutputFileRequest { data_id };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_output_file(request);
    assert!(response.is_err());
}

fn test_get_fusion_data() {
    let mut client = get_client();

    let request = GetFusionDataRequest {
        data_id: "fusion-data-mock-frontend-data".to_string(),
    };
    let response = client.get_fusion_data(request);
    assert!(response.is_ok());
    assert!(response.unwrap().hash.is_empty());

    let request = GetFusionDataRequest {
        data_id: "fusion-data-mock-frontend-data".to_string(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_fusion_data(request);
    assert!(response.is_err());
}

fn test_register_function() {
    let mut client = get_client();

    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: true,
        arg_list: vec!["arg".to_string()],
        input_list: vec![],
        output_list: vec![],
    };
    let response = client.register_function(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().function_id.is_empty());

    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: true,
        arg_list: vec!["arg".to_string()],
        input_list: vec![],
        output_list: vec![],
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_function(request);
    assert!(response.is_err());
}

fn test_get_function() {
    let mut client = get_client();

    let request = GetFunctionRequest {
        function_id: "native-mock-simple-func".to_string(),
    };
    let response = client.get_function(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().name.is_empty());

    let request = GetFunctionRequest {
        function_id: "native-mock-simple-func".to_string(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_function(request);
    assert!(response.is_err());
}

fn test_create_task() {
    let mut client = get_client();

    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["frontend_user".to_string(), "mock_user".to_string()]
            .into_iter()
            .collect(),
    };
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list: output_data_owner_list.clone(),
    };
    let response = client.create_task(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().task_id.is_empty());

    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list,
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.create_task(request);
    assert!(response.is_err());
}

fn test_get_task() {
    let mut client = get_client();

    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["frontend_user".to_string(), "mock_user".to_string()]
            .into_iter()
            .collect(),
    };
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list,
    };
    let response = client.create_task(request);
    let task_id = response.unwrap().task_id;

    let request = GetTaskRequest {
        task_id: task_id.clone(),
    };
    let response = client.get_task(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().function_id.is_empty());

    let request = GetTaskRequest { task_id };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_task(request);
    assert!(response.is_err());
}

fn test_assign_data() {
    let mut client = get_client();

    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["frontend_user".to_string()].into_iter().collect(),
    };
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list,
    };
    let response = client.create_task(request);
    let task_id = response.unwrap().task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client.register_output_file(request);
    let output_id = response.unwrap().data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: vec![("output".to_string(), output_id.clone())]
            .into_iter()
            .collect(),
    };
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.assign_data(request);
    assert!(response.is_err());

    let request = AssignDataRequest {
        task_id,
        input_map: HashMap::new(),
        output_map: vec![("output".to_string(), output_id)]
            .into_iter()
            .collect(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.assign_data(request);
    assert!(response.is_ok());
}

fn test_approve_task() {
    let mut client = get_client();

    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["frontend_user".to_string()].into_iter().collect(),
    };
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list,
    };
    let response = client.create_task(request);
    let task_id = response.unwrap().task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client.register_output_file(request);
    let output_id = response.unwrap().data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: vec![("output".to_string(), output_id)]
            .into_iter()
            .collect(),
    };
    let _response = client.assign_data(request);

    let request = ApproveTaskRequest {
        task_id: task_id.clone(),
    };
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.approve_task(request);
    assert!(response.is_err());

    let request = ApproveTaskRequest { task_id };
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.approve_task(request);
    assert!(response.is_ok());
}

fn test_invoke_task() {
    let mut client = get_client();

    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["frontend_user".to_string()].into_iter().collect(),
    };
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    let request = CreateTaskRequest {
        function_id: "native-mock-simple-func".to_string(),
        arg_list: vec![("arg1".to_string(), "data1".to_string())]
            .into_iter()
            .collect(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list,
    };
    let response = client.create_task(request);
    let task_id = response.unwrap().task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client.register_output_file(request);
    let output_id = response.unwrap().data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: vec![("output".to_string(), output_id)]
            .into_iter()
            .collect(),
    };
    let _response = client.assign_data(request);

    let request = ApproveTaskRequest {
        task_id: task_id.clone(),
    };
    let _response = client.approve_task(request);

    let request = InvokeTaskRequest {
        task_id: task_id.clone(),
    };
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.invoke_task(request);
    assert!(response.is_err());

    let request = InvokeTaskRequest {
        task_id: task_id.clone(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.invoke_task(request);
    assert!(response.is_ok());

    let request = GetTaskRequest { task_id };
    let response = client.get_task(request);
    assert_eq!(response.unwrap().status, TaskStatus::Running);
}
