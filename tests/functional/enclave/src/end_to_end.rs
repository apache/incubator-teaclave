use anyhow::Result;
use lazy_static::lazy_static;
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

static USERNAME: &'static str = "alice";
static PASSWORD: &'static str = "daHosldOdker0sS";
static CONFIG_FILE: &'static str = "runtime.config.toml";
static AUTH_SERVICE_ADDR: &'static str = "localhost:7776";
static FRONTEND_SERVICE_ADDR: &'static str = "localhost:7777";

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;
    setup();
    run_tests!(test_echo_task,)
}

lazy_static! {
    static ref ENCLAVE_INFO: EnclaveInfo = {
        let runtime_config = RuntimeConfig::from_toml(CONFIG_FILE).expect("runtime config");
        EnclaveInfo::from_bytes(
            &runtime_config
                .audit
                .enclave_info_bytes
                .as_ref()
                .expect("encalve info"),
        )
    };
}

fn setup() {
    // Register user for the first time
    let mut api_client =
        create_authentication_api_client(&ENCLAVE_INFO, AUTH_SERVICE_ADDR).unwrap();
    register_new_account(&mut api_client, USERNAME, PASSWORD).unwrap();
}

fn test_echo_task() {
    // Authenticate user before talking to frontend service
    let mut api_client =
        create_authentication_api_client(&ENCLAVE_INFO, AUTH_SERVICE_ADDR).unwrap();
    let cred = login(&mut api_client, USERNAME, PASSWORD).unwrap();
    let mut client = create_frontend_client(&ENCLAVE_INFO, FRONTEND_SERVICE_ADDR, cred).unwrap();

    // Register Function
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

    // Create Task
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

    // Assign Data To Task
    let task_id = response.task_id;
    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    let response = client.assign_data(request).unwrap();

    log::info!("Assign data: {:?}", response);

    // Approve Task
    let request = ApproveTaskRequest::new(&task_id);
    let response = client.approve_task(request).unwrap();

    log::info!("Approve task: {:?}", response);

    // Invoke Task
    let request = InvokeTaskRequest::new(&task_id);
    let response = client.invoke_task(request).unwrap();

    log::info!("Invoke task: {:?}", response);

    // Get Task
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

fn create_client_config(
    enclave_info: &EnclaveInfo,
    service_name: &str,
) -> Result<SgxTrustedTlsClientConfig> {
    let enclave_attr = enclave_info
        .get_enclave_attr(service_name)
        .expect("enclave attr");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );
    Ok(config)
}

fn create_frontend_client(
    enclave_info: &EnclaveInfo,
    service_addr: &str,
    cred: UserCredential,
) -> Result<TeaclaveFrontendClient> {
    let tls_config = create_client_config(&enclave_info, "teaclave_frontend_service")?;
    let channel = Endpoint::new(service_addr).config(tls_config).connect()?;

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), cred.id);
    metadata.insert("token".to_string(), cred.token);

    let client = TeaclaveFrontendClient::new_with_metadata(channel, metadata)?;
    Ok(client)
}

fn create_authentication_api_client(
    enclave_info: &EnclaveInfo,
    service_addr: &str,
) -> Result<TeaclaveAuthenticationApiClient> {
    let tls_config = create_client_config(&enclave_info, "teaclave_authentication_service")?;
    let channel = Endpoint::new(service_addr).config(tls_config).connect()?;

    let client = TeaclaveAuthenticationApiClient::new(channel)?;
    Ok(client)
}

fn register_new_account(
    api_client: &mut TeaclaveAuthenticationApiClient,
    username: &str,
    password: &str,
) -> Result<()> {
    let request = UserRegisterRequest::new(username, password);
    let response = api_client.user_register(request)?;

    log::info!("User register: {:?}", response);

    Ok(())
}

fn login(
    api_client: &mut TeaclaveAuthenticationApiClient,
    username: &str,
    password: &str,
) -> Result<UserCredential> {
    let request = UserLoginRequest::new(username, password);
    let response = api_client.user_login(request)?;

    log::info!("User login: {:?}", response);

    Ok(UserCredential::new(username, response.token))
}
