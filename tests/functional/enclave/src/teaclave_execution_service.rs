use std::prelude::v1::*;

use std::convert::TryInto;
use teaclave_proto::teaclave_execution_service::*;
use teaclave_rpc::endpoint::Endpoint;

use teaclave_config::RuntimeConfig;
use teaclave_types::convert_plaintext_file;
use teaclave_types::hashmap;
use teaclave_types::TeaclaveFileCryptoInfo;
use teaclave_types::TeaclaveFunctionArguments;
use teaclave_types::TeaclaveWorkerFileInfo;
use teaclave_types::TeaclaveWorkerFileRegistry;
use teaclave_types::WorkerInvocation;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;
    run_tests!(test_invoke_success,)
}

fn get_client() -> TeaclaveExecutionClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let channel = Endpoint::new(
        &runtime_config
            .internal_endpoints
            .execution
            .advertised_address,
    )
    .connect()
    .expect("channel");
    TeaclaveExecutionClient::new(channel).expect("client")
}

fn test_invoke_success() {
    let mut client = get_client();

    let function_args = TeaclaveFunctionArguments::new(&hashmap!(
        "feature_size"  => "4",
        "max_depth"     => "4",
        "iterations"    => "100",
        "shrinkage"     => "0.1",
        "feature_sample_ratio" => "1.0",
        "data_sample_ratio" => "1.0",
        "min_leaf_size" => "1",
        "loss"          => "LAD",
        "training_optimization_level" => "2"
    ));

    let enc_input = "fixtures/functions/gbdt_training/train.enc";
    let plain_input = "fixtures/functions/gbdt_training/train.txt";
    let enc_output = "fixtures/functions/gbdt_training/model.enc.out";

    let input_info = convert_plaintext_file(plain_input, enc_input).unwrap();
    let input_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "training_data".to_string() => input_info));

    let output_info = TeaclaveWorkerFileInfo::new(enc_output, TeaclaveFileCryptoInfo::default());
    let output_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "trained_model".to_string() => output_info));

    let request = StagedFunctionExecuteRequest {
        invocation: WorkerInvocation {
            runtime_name: "default".to_string(),
            executor_type: "native".try_into().unwrap(),
            function_name: "gbdt_training".to_string(),
            function_payload: String::new(),
            function_args,
            input_files,
            output_files,
        },
    };

    let response_result = client.invoke_function(request);
    assert!(response_result.is_ok());
}
