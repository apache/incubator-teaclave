use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_proto::teaclave_management_service::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;
use url::Url;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(
        test_register_input_file,
        test_register_output_file,
        test_register_function,
        test_create_task,
        test_get_output_file,
        test_get_fusion_data,
        test_get_function,
        test_get_task,
    )
}

fn get_client(user_id: &str) -> TeaclaveManagementClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_management_service")
        .expect("management");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new(
        &runtime_config
            .internal_endpoints
            .management
            .advertised_address,
    )
    .config(config)
    .connect()
    .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), user_id.to_string());
    metadata.insert("token".to_string(), "".to_string());

    TeaclaveManagementClient::new_with_metadata(channel, metadata).unwrap()
}
fn test_register_input_file() {
    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::AesGcm128(AesGcm128CryptoInfo {
            key: [0x90u8; 16],
            iv: [0x89u8; 12],
        }),
    };

    let mut client = get_client("mock_user");
    let response = client.register_input_file(request);

    assert!(response.is_ok());
}

fn test_register_output_file() {
    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::AesGcm128(AesGcm128CryptoInfo {
            key: [0x90u8; 16],
            iv: [0x89u8; 12],
        }),
    };

    let mut client = get_client("mock_user");
    let response = client.register_output_file(request);

    assert!(response.is_ok());
}

fn test_get_output_file() {
    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::AesGcm128(AesGcm128CryptoInfo {
            key: [0x90u8; 16],
            iv: [0x89u8; 12],
        }),
    };

    let mut client = get_client("mock_user");
    let response = client.register_output_file(request).unwrap();
    let data_id = response.data_id;
    let request = GetOutputFileRequest {
        data_id: data_id.clone(),
    };
    let response = client.get_output_file(request);
    assert!(response.is_ok());
    let mut client = get_client("mock_another_user");
    let request = GetOutputFileRequest { data_id };
    let response = client.get_output_file(request);
    assert!(response.is_err());
}

fn test_get_fusion_data() {
    let mut client = get_client("mock_user_a");
    let request = GetFusionDataRequest {
        data_id: "fusion-data-mock-data".to_string(),
    };
    let response = client.get_fusion_data(request);
    assert!(response.is_ok());

    let mut client = get_client("mock_user_b");
    let request = GetFusionDataRequest {
        data_id: "fusion-data-mock-data".to_string(),
    };
    let response = client.get_fusion_data(request);
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.hash.is_empty());
    assert_eq!(
        response.data_owner_id_list,
        ["mock_user_a".to_string(), "mock_user_b".to_string()]
    );

    let mut client = get_client("mock_user_c");
    let request = GetFusionDataRequest {
        data_id: "fusion-data-mock-data".to_string(),
    };
    let response = client.get_fusion_data(request);
    assert!(response.is_err());
}

fn test_register_function() {
    let function_input = FunctionInput {
        name: "input".to_string(),
        description: "input_desc".to_string(),
    };
    let function_output = FunctionOutput {
        name: "output".to_string(),
        description: "output_desc".to_string(),
    };
    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: true,
        arg_list: vec!["arg".to_string()],
        input_list: vec![function_input],
        output_list: vec![function_output],
    };

    let mut client = get_client("mock_user");
    let response = client.register_function(request);

    assert!(response.is_ok());
}

fn test_get_function() {
    let function_input = FunctionInput {
        name: "input".to_string(),
        description: "input_desc".to_string(),
    };
    let function_output = FunctionOutput {
        name: "output".to_string(),
        description: "output_desc".to_string(),
    };
    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: false,
        arg_list: vec!["arg".to_string()],
        input_list: vec![function_input],
        output_list: vec![function_output],
    };

    let mut client = get_client("mock_user");
    let response = client.register_function(request);
    let function_id = response.unwrap().function_id;

    let request = GetFunctionRequest {
        function_id: function_id.clone(),
    };
    let response = client.get_function(request);
    assert!(response.is_ok());
    info!("{:?}", response.unwrap());
    let mut client = get_client("mock_unauthorized_user");
    let request = GetFunctionRequest { function_id };
    let response = client.get_function(request);
    assert!(response.is_err());
    let request = GetFunctionRequest {
        function_id: "native-mock-native-func".to_string(),
    };
    let response = client.get_function(request);
    assert!(response.is_ok());
    info!("{:?}", response.unwrap());
}

fn get_correct_create_task() -> CreateTaskRequest {
    let mut arg_list = HashMap::new();
    arg_list.insert("arg1".to_string(), "data1".to_string());
    arg_list.insert("arg2".to_string(), "data2".to_string());
    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["mock_user1".to_string()].into_iter().collect(),
    };
    let data_owner_id_list2 = DataOwnerList {
        user_id_list: vec!["mock_user2".to_string(), "mock_user3".to_string()]
            .into_iter()
            .collect(),
    };
    let mut input_data_owner_list = HashMap::new();
    input_data_owner_list.insert("input".to_string(), data_owner_id_list.clone());
    input_data_owner_list.insert("input2".to_string(), data_owner_id_list2.clone());
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    output_data_owner_list.insert("output2".to_string(), data_owner_id_list2);

    CreateTaskRequest {
        function_id: "native-mock-native-func".to_string(),
        arg_list,
        input_data_owner_list,
        output_data_owner_list,
    }
}

fn test_create_task() {
    let request = CreateTaskRequest {
        function_id: "invalid_function".to_string(),
        arg_list: HashMap::new(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list: HashMap::new(),
    };
    let mut client = get_client("mock_user");
    let response = client.create_task(request);
    assert!(response.is_err());

    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());

    let mut request = get_correct_create_task();
    request.arg_list.remove("arg1");
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = get_correct_create_task();
    request.input_data_owner_list.remove("input");
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = get_correct_create_task();
    request.output_data_owner_list.remove("output");
    let response = client.create_task(request);
    assert!(response.is_err());
}

fn test_get_task() {
    let mut client = get_client("mock_user");
    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    let request = GetTaskRequest { task_id };
    let response = client.get_task(request);
    assert!(response.is_ok());
    let response = response.unwrap();
    info!("{:?}", response);
    let participants = vec!["mock_user1", "mock_user3", "mock_user2", "mock_user"];
    for name in participants {
        assert!(response.participants.contains(&name.to_string()));
    }
    assert!(response.participants.len() == 4);
    assert!(response.output_map.len() == 1);
}
