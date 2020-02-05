use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::EnclaveInfo;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(
        test_authorize_data_success,
        test_authorize_data_fail,
        test_authorize_function_success,
        test_authorize_function_fail,
        test_authorize_task_success,
        test_authorize_task_fail,
        test_authorize_staged_task_success,
        test_authorize_staged_task_fail,
        test_concurrency,
    )
}

fn get_client() -> TeaclaveAccessControlClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_access_control_service")
        .expect("access_control");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.ias_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new(
        &runtime_config
            .internal_endpoints
            .access_control
            .advertised_address,
    )
    .config(config)
    .connect()
    .unwrap();
    TeaclaveAccessControlClient::new(channel).unwrap()
}

fn test_authorize_data_success() {
    let mut client = get_client();

    let request = AuthorizeDataRequest::new("mock_user_a", "mock_data");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

fn test_authorize_data_fail() {
    let mut client = get_client();

    let request = AuthorizeDataRequest::new("mock_user_d", "mock_data");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeDataRequest::new("mock_user_a", "mock_data_b");
    let response_result = client.authorize_data(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

fn test_authorize_function_success() {
    let mut client = get_client();

    let request =
        AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_public_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request =
        AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_private_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request =
        AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_public_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

fn test_authorize_function_fail() {
    let mut client = get_client();
    let request =
        AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_private_function");
    let response_result = client.authorize_function(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

fn test_authorize_task_success() {
    let mut client = get_client();
    let request = AuthorizeTaskRequest::new("mock_participant_a", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);

    let request = AuthorizeTaskRequest::new("mock_participant_b", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

fn test_authorize_task_fail() {
    let mut client = get_client();
    let request = AuthorizeTaskRequest::new("mock_participant_c", "mock_task");
    let response_result = client.authorize_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

fn test_authorize_staged_task_success() {
    let mut client = get_client();
    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec![
            "mock_staged_allowed_data1".to_string(),
            "mock_staged_allowed_data2".to_string(),
            "mock_staged_allowed_data3".to_string(),
        ],
        object_output_data_id_list: vec![
            "mock_staged_allowed_data1".to_string(),
            "mock_staged_allowed_data2".to_string(),
            "mock_staged_allowed_data3".to_string(),
        ],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(response_result.unwrap().accept);
}

fn test_authorize_staged_task_fail() {
    let mut client = get_client();
    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_disallowed_private_function".to_string(),
        object_input_data_id_list: vec![],
        object_output_data_id_list: vec![],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec!["mock_staged_disallowed_data1".to_string()],
        object_output_data_id_list: vec![],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);

    let request = AuthorizeStagedTaskRequest {
        subject_task_id: "mock_staged_task".to_string(),
        object_function_id: "mock_staged_allowed_private_function".to_string(),
        object_input_data_id_list: vec![],
        object_output_data_id_list: vec!["mock_staged_disallowed_data2".to_string()],
    };
    let response_result = client.authorize_staged_task(request);
    assert!(response_result.is_ok());
    assert!(!response_result.unwrap().accept);
}

fn test_concurrency() {
    let mut thread_pool = Vec::new();
    for _i in 0..10 {
        let child = std::thread::spawn(move || {
            for _j in 0..10 {
                test_authorize_data_fail();
                test_authorize_function_fail();
                test_authorize_task_success();
                test_authorize_staged_task_fail();
            }
        });
        thread_pool.push(child);
    }
    for thr in thread_pool.into_iter() {
        assert!(thr.join().is_ok());
    }
}
