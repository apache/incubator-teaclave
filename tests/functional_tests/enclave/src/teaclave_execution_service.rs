use anyhow;
use serde_json;
use std::io::Write;
use std::prelude::v1::*;
use std::untrusted::fs;

use teaclave_proto::teaclave_execution_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::AesGcm128CryptoInfo;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;
    run_tests!(test_invoke_success,)
}

fn setup_client() -> TeaclaveExecutionClient {
    let channel = Endpoint::new("localhost:7989").connect().unwrap();
    TeaclaveExecutionClient::new(channel).unwrap()
}

fn enc_input_file(input_crypto: &str, plain_input: &str, enc_input: &str) -> anyhow::Result<()> {
    let crypto_info: AesGcm128CryptoInfo = serde_json::from_str(input_crypto)?;
    let mut bytes = fs::read_to_string(plain_input)?.into_bytes();
    crypto_info.encrypt(&mut bytes)?;

    let mut file = fs::File::create(enc_input)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn test_invoke_success() {
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
                "path": "test_cases/gbdt_training/model.enc.out",
                "crypto_info": {
                    "teaclave_file_root_key128": {
                        "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
                    }
                }
            }
        }
    }"#;

    let input_crypto = r#"{
            "key": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            "iv": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
    }"#;
    let enc_input = "test_cases/gbdt_training/train.enc";
    let plain_input = "test_cases/gbdt_training/train.txt";

    enc_input_file(input_crypto, plain_input, enc_input).unwrap();
    let request: StagedFunctionExecuteRequest = serde_json::from_str(request_payload).unwrap();
    let mut client = setup_client();
    let response_result = client.invoke_function(request);
    assert!(response_result.is_ok());
}
