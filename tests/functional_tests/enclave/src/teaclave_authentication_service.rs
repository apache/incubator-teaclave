use sgx_tunittest::*;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::build_config::BUILD_CONFIG;
use teaclave_config::runtime_config::RuntimeConfig;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::EnclaveInfo;

pub fn run_tests() {
    rsgx_unit_tests!(
        test_login_success,
        test_login_fail,
        test_authorize_success,
        test_authorize_fail,
    );
}

fn test_login_success() {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let measure = enclave_info
        .measurements
        .get("teaclave_authentication_service")
        .expect("authentication");
    let enclave_attr = verifier::EnclaveAttr {
        measures: vec![*measure],
    };
    let config = SgxTrustedTlsClientConfig::new_with_attestation_report_verifier(
        enclave_attr,
        BUILD_CONFIG.ias_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new("localhost:7776")
        .config(config)
        .connect()
        .unwrap();
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

fn test_login_fail() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
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

fn test_authorize_success() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
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

fn test_authorize_fail() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
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
