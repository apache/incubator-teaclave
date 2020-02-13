use std::convert::TryInto;
use std::io::Read;
use std::prelude::v1::*;

use teaclave_types::convert_plaintext_file;
use teaclave_types::hashmap;
use teaclave_types::read_all_bytes;
use teaclave_types::TeaclaveFileCryptoInfo;
use teaclave_types::TeaclaveFunctionArguments;
use teaclave_types::TeaclaveWorkerFileInfo;
use teaclave_types::TeaclaveWorkerFileRegistry;
use teaclave_types::WorkerInvocation;

use teaclave_worker::Worker;

fn test_start_worker() {
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
    let expected_output = "fixtures/functions/gbdt_training/expected_model.txt";

    let input_info = convert_plaintext_file(plain_input, enc_input).unwrap();

    let input_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "training_data".to_string() => input_info));

    let output_info = TeaclaveWorkerFileInfo::new(enc_output, TeaclaveFileCryptoInfo::default());

    let output_files = TeaclaveWorkerFileRegistry::new(hashmap!(
        "trained_model".to_string() => output_info.clone()));

    let request = WorkerInvocation {
        runtime_name: "default".to_string(),
        executor_type: "native".try_into().unwrap(),
        function_name: "gbdt_training".to_string(),
        function_payload: String::new(),
        function_args,
        input_files,
        output_files,
    };

    let worker = Worker::default();
    let summary = worker.invoke_function(request).unwrap();
    assert_eq!(summary, "Trained 120 lines of data.");

    let mut f = output_info.get_readable_io().unwrap();
    let mut result = Vec::new();
    f.read_to_end(&mut result).unwrap();

    let expected = read_all_bytes(expected_output).unwrap();
    assert_eq!(&result[..], &expected[..]);
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_start_worker)
}
