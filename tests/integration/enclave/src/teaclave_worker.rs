use std::prelude::v1::*;

use teaclave_types::{
    hashmap, read_all_bytes, ExecutorType, FunctionArguments, StagedFunction,
    TeaclaveFileRootKey128, TeaclaveWorkerFileRegistry, TeaclaveWorkerInputFileInfo,
    TeaclaveWorkerOutputFileInfo,
};
use teaclave_worker::Worker;

fn test_start_worker() {
    let arguments = FunctionArguments::from_map(&hashmap!(
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

    let plain_input = "fixtures/functions/gbdt_training/train.txt";
    let enc_output = "fixtures/functions/gbdt_training/model.enc.out";
    let expected_output = "fixtures/functions/gbdt_training/expected_model.txt";

    let input_info = TeaclaveWorkerInputFileInfo::create_with_plaintext_file(plain_input).unwrap();

    let input_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "training_data".to_string() => input_info));

    let output_info =
        TeaclaveWorkerOutputFileInfo::new(enc_output, TeaclaveFileRootKey128::random());

    let output_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "trained_model".to_string() => output_info.clone()));

    let request = StagedFunction {
        name: "gbdt_training".to_string(),
        payload: String::new(),
        arguments,
        input_files,
        output_files,
        runtime_name: "default".to_string(),
        executor_type: ExecutorType::Native,
    };

    let worker = Worker::default();

    let capability = worker.get_capability();
    assert!(capability.runtimes.contains("default"));
    assert!(capability.functions.contains("native-gbdt_training"));

    let summary = worker.invoke_function(request).unwrap();
    assert_eq!(summary, "Trained 120 lines of data.");

    let result = output_info.get_plaintext().unwrap();
    let expected = read_all_bytes(expected_output).unwrap();
    assert_eq!(&result[..], &expected[..]);
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_start_worker)
}
