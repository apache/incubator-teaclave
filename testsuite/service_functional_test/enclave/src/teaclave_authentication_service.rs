use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_rpc::channel::SgxTrustedTlsChannel;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_types::TeaclaveServiceResponseResult;

type U = TeaclaveAuthenticationRequest;
type V = TeaclaveServiceResponseResult<TeaclaveAuthenticationResponse>;

pub fn run_functional_tests() {
    rsgx_unit_tests!(
        test_login_successful,
        test_login_failed,
        test_authorize_successful,
        test_authorize_failed,
    );
}

fn test_login_successful() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel = SgxTrustedTlsChannel::<U, V>::new("localhost:7776", &client_config).unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserLoginRequest {
        id: "test_id".to_string(),
        password: "test_password".to_string(),
    }
    .into();
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_login_failed() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel = SgxTrustedTlsChannel::<U, V>::new("localhost:7776", &client_config).unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserLoginRequest {
        id: "".to_string(),
        password: "".to_string(),
    }
    .into();
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_err());
}

fn test_authorize_successful() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel = SgxTrustedTlsChannel::<U, V>::new("localhost:7776", &client_config).unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let credential = UserCredential {
        id: "test_id".to_string(),
        token: "test_token".to_string(),
    };
    let request = UserAuthorizeRequest { credential }.into();
    let response_result = client.user_authorize(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_authorize_failed() {
    let client_config = SgxTrustedTlsClientConfig::new_without_verifier();
    let channel = SgxTrustedTlsChannel::<U, V>::new("localhost:7776", &client_config).unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let credential = UserCredential {
        id: "".to_string(),
        token: "".to_string(),
    };
    let request = UserAuthorizeRequest { credential }.into();
    let response_result = client.user_authorize(request);
    info!("{:?}", response_result);
    assert!(response_result.is_err());
}
