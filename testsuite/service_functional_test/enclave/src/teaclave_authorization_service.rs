use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_proto::teaclave_authorization_service::proto::*;
use teaclave_rpc::channel::SgxTrustedTlsChannel;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_types::TeaclaveServiceResponseResult;

type U = TeaclaveAuthorizationRequest;
type V = TeaclaveServiceResponseResult<TeaclaveAuthorizationResponse>;

pub fn run_functional_tests() {
    rsgx_unit_tests!(test_login_success, test_login_failed,);
}

fn test_login_success() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel =
        SgxTrustedTlsChannel::<U, V>::new("127.0.0.1:7776", "localhost", &client_config).unwrap();
    let mut client = TeaclaveAuthorizationClient::new(channel).unwrap();
    let request = UserLoginRequest {
        id: "test_id".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_login_failed() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel =
        SgxTrustedTlsChannel::<U, V>::new("127.0.0.1:7776", "localhost", &client_config).unwrap();
    let mut client = TeaclaveAuthorizationClient::new(channel).unwrap();
    let request = UserLoginRequest {
        id: "".to_string(),
        password: "".to_string(),
    };
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_err());
}
