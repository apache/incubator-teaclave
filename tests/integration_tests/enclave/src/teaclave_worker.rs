use std::io::Write;
use std::prelude::v1::*;
use std::untrusted::fs;

use serde_json;

use anyhow;
use teaclave_types::AesGcm128CryptoInfo;
use teaclave_types::WorkerInvocation;
use teaclave_worker::Worker;

fn enc_input_file() -> anyhow::Result<()> {
    let crypto_info_str = r#"{
            "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    }"#;
    let crypto_info: AesGcm128CryptoInfo = serde_json::from_str(crypto_info_str)?;

    let plain_input = "test_cases/gbdt_training/train.txt";
    let enc_input = "test_cases/gbdt_training/train.enc";
    let mut bytes = fs::read_to_string(plain_input)?.into_bytes();
    crypto_info.encrypt(&mut bytes)?;

    let mut file = fs::File::create(enc_input)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn test_start_worker() {
    let request_payload = r#"{
        "runtime_name": "default",
        "executor_type": "native",
        "function_name": "gbdt_training",
        "function_payload": "",
        "function_args": {
            "feature_size": "4",
            "max_depth": "4",
            "iterations": "100",
            "shrinkage": "0.1",
            "feature_sample_ratio": "1.0",
            "data_sample_ratio": "1.0",
            "min_leaf_size": "1",
            "loss": "LAD",
            "training_optimization_level": "2"
        },
        "input_files": {
            "training_data": {
                "path": "test_cases/gbdt_training/train.enc",
                "crypto_info": {
                    "aes_gcm128": {
                        "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                        "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
                    }
                }
            }
        },
        "output_files": {
            "trained_model": {
                "path": "test_cases/gbdt_training/model.txt.out",
                "crypto_info": {
                    "aes_gcm128": {
                        "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                        "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
                    }
                }
            }
        }
    }"#;

    enc_input_file().unwrap();
    let plain_output = "test_cases/gbdt_training/model.txt.out";
    let expected_output = "test_cases/gbdt_training/expected_model.txt";
    let request: WorkerInvocation = serde_json::from_str(request_payload).unwrap();
    let worker = Worker::default();
    let summary = worker.invoke_function(request).unwrap();
    assert_eq!(summary, "Trained 120 lines of data.");

    let result = fs::read_to_string(&plain_output).unwrap();
    let expected = fs::read_to_string(&expected_output).unwrap();
    assert_eq!(&result[..], &expected[..]);
}

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_start_worker)
}
