use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::EnclaveInfo;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(
        test_login_success,
        test_login_fail,
        test_authenticate_success,
        test_authenticate_fail,
        test_register_success,
        test_register_fail,
    )
}

fn test_login_success() {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_authentication_service")
        .expect("authentication");
    let config = SgxTrustedTlsClientConfig::new_with_attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.ias_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new("localhost:7776")
        .config(config)
        .connect()
        .unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_login_id1".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    assert!(response_result.is_ok());

    let request = UserLoginRequest {
        id: "test_login_id1".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_login_fail() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_login_id2".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    assert!(response_result.is_ok());

    let request = UserLoginRequest {
        id: "test_login_id2".to_string(),
        password: "wrong_password".to_string(),
    };
    let response_result = client.user_login(request);
    info!("{:?}", response_result);
    assert!(response_result.is_err());
}

fn test_authenticate_success() {
    let channel = Endpoint::new("localhost:17776").connect().unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_authenticate_id1".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    assert!(response_result.is_ok());

    let request = UserLoginRequest {
        id: "test_authenticate_id1".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_login(request);
    assert!(response_result.is_ok());
    let credential = UserCredential {
        id: "test_authenticate_id1".to_string(),
        token: response_result.unwrap().token,
    };
    let request = UserAuthenticateRequest { credential };
    let response_result = client.user_authenticate(request);
    info!("{:?}", response_result);
    assert!(response_result.unwrap().accept);
}

fn test_authenticate_fail() {
    let channel = Endpoint::new("localhost:17776").connect().unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_authenticate_id2".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    assert!(response_result.is_ok());

    let request = UserLoginRequest {
        id: "test_authenticate_id2".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_login(request);
    assert!(response_result.is_ok());
    let credential = UserCredential {
        id: "test_authenticate_id2".to_string(),
        token: "wrong_token".to_string(),
    };
    let request = UserAuthenticateRequest { credential };
    let response_result = client.user_authenticate(request);
    info!("{:?}", response_result);
    assert!(!response_result.unwrap().accept);
}

fn test_register_success() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_register_id1".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    info!("{:?}", response_result);
    assert!(response_result.is_ok());
}

fn test_register_fail() {
    let channel = Endpoint::new("localhost:7776").connect().unwrap();
    let mut client = TeaclaveAuthenticationClient::new(channel).unwrap();
    let request = UserRegisterRequest {
        id: "test_register_id2".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    assert!(response_result.is_ok());
    let request = UserRegisterRequest {
        id: "test_register_id2".to_string(),
        password: "test_password".to_string(),
    };
    let response_result = client.user_register(request);
    info!("{:?}", response_result);
    assert!(response_result.is_err());
}
