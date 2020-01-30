use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_register_input_file_authentication_error)
}

fn test_register_input_file_authentication_error() {
    let request = RegisterInputFileRequest {
        uri: "".to_string(),
        hash: "".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::AesGcm128(AesGcm128CryptoInfo {
            key: [0x90u8; 16],
            iv: [0x89u8; 12],
        }),
        credential: UserCredential {
            id: "".to_string(),
            token: "".to_string(),
        },
    };

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), "".to_string());
    metadata.insert("token".to_string(), "".to_string());

    let channel = Endpoint::new("localhost:7777").connect().unwrap();
    let mut client = TeaclaveFrontendClient::new_with_metadata(channel, metadata).unwrap();
    let response = client.register_input_file(request);

    assert_eq!(
        response,
        Err(TeaclaveServiceResponseError::RequestError(
            "authentication error".to_string()
        ))
    );
}
