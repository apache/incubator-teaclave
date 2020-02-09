use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;
use url::Url;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_register_input_file_authentication_error)
}

fn get_client() -> TeaclaveFrontendClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let port = &runtime_config.api_endpoints.frontend.listen_address.port();
    let channel = Endpoint::new(&format!("localhost:{}", port))
        .connect()
        .unwrap();
    TeaclaveFrontendClient::new(channel).unwrap()
}

fn test_register_input_file_authentication_error() {
    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::AesGcm128(AesGcm128CryptoInfo {
            key: [0x90u8; 16],
            iv: [0x89u8; 12],
        }),
    };

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), "".to_string());
    metadata.insert("token".to_string(), "".to_string());

    let mut client = get_client();
    let response = client.register_input_file(request);

    assert_eq!(
        response,
        Err(TeaclaveServiceResponseError::RequestError(
            "authentication error".to_string()
        ))
    );
}
