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

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_echo_task,)
}

fn get_credential() -> UserCredential {
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

fn get_frontend_client() -> TeaclaveFrontendClient {
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

fn test_echo_task() {
    let mut client = get_frontend_client();

    let request = RegisterFunctionRequest {
        name: "echo".to_string(),
        description: "Native Echo Function".to_string(),
        payload: vec![],
        is_public: true,
        arg_list: vec!["message".to_string()],
        input_list: vec![],
        output_list: vec![],
    };
    let response = client.register_function(request).unwrap();

    log::info!("Resgister function: {:?}", response);

    let function_id = response.function_id;
    let function_arguments = FunctionArguments::new(hashmap!("message" => "Hello From Teaclave!"));
    let request = CreateTaskRequest {
        function_id,
        function_arguments,
        input_data_owner_list: HashMap::new(),
        output_data_owner_list: HashMap::new(),
    };
    let response = client.create_task(request).unwrap();

    log::info!("Create task: {:?}", response);

    let task_id = response.task_id;
    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    let response = client.assign_data(request).unwrap();

    log::info!("Assign data: {:?}", response);

    let request = ApproveTaskRequest::new(&task_id);
    let response = client.approve_task(request).unwrap();

    log::info!("Approve task: {:?}", response);

    let request = InvokeTaskRequest::new(&task_id);
    let response = client.invoke_task(request).unwrap();

    log::info!("Invoke task: {:?}", response);

    loop {
        let request = GetTaskRequest::new(&task_id);
        let response = client.get_task(request).unwrap();
        log::info!("Get task: {:?}", response);
        std::thread::sleep(std::time::Duration::from_secs(1));
        if response.status != TaskStatus::Running {
            let ret_val = String::from_utf8(response.return_value.unwrap()).unwrap();
            log::info!("Task returns: {:?}", ret_val);
            assert_eq!(&ret_val, "Hello From Teaclave!");
            break;
        }
    }
}
