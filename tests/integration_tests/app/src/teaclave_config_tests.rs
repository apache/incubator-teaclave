use teaclave_config;

pub fn run_tests() {
    teaclave_test_utils::tests!(runtime_config);
}

fn runtime_config() {
    let config =
        teaclave_config::runtime_config::RuntimeConfig::from_toml("./fixtures/runtime.config.toml")
            .unwrap();
    assert_eq!(
        config.api_endpoints.authentication.listen_address,
        "0.0.0.0:7776".parse().unwrap()
    );
    assert!(config.ias.as_ref().unwrap().ias_key == "ias_key_AAAABBBBCCCCDDDDEEEEFFFF");
    assert!(config.ias.as_ref().unwrap().ias_spid == "ias_spid_AAAABBBBCCCCDDDDEEEEFFF");

    assert!(config.audit.auditor_signatures_bytes.as_ref().unwrap()[0] == b"godzilla.sign.sha256")
}
