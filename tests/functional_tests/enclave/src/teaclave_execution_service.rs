use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_proto::teaclave_execution_service::*;
use teaclave_rpc::endpoint::Endpoint;
use serde_json;

pub fn run_tests() {
    rsgx_unit_tests!(
        test_get_success,
        //test_get_fail,
        //test_put_success,
        //test_delete_success,
        //test_enqueue_success,
        //test_dequeue_success,
        //test_dequeue_fail,
    );
}

fn setup_client() -> TeaclaveExecutionClient {
    let channel = Endpoint::new("localhost:7989").connect().unwrap();
    TeaclaveExecutionClient::new(channel).unwrap()
}

fn test_get_success() {
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
                "path": "test_cases/gbdt_training/train.txt",
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

    let request: StagedFunctionExecuteRequest =
        serde_json::from_str(request_payload).unwrap();

    let mut client = setup_client();
    let response_result = client.invoke_function(request.into());
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}