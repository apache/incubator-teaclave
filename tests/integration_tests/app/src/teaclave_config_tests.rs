use std::env;
use teaclave_config;

pub fn run_tests() {
    teaclave_test_utils::tests!(runtime_config, runtime_config_with_env_vars);
}

fn runtime_config() {
    env::remove_var("IAS_KEY");
    env::remove_var("IAS_SPID");
    let config =
        teaclave_config::RuntimeConfig::from_toml("./fixtures/runtime.config.toml").unwrap();
    let authentication_config = config.api_endpoints.authentication;
    assert_eq!(authentication_config.listen_address, "0.0.0.0:7776");
    let dbs_config = config.internal_endpoints.dbs;
    assert_eq!(dbs_config.listen_address, "0.0.0.0:7778");
    assert_eq!(
        dbs_config.inbound_services,
        Some(vec!["frontend".to_string()])
    );

    assert_eq!(
        config.ias.as_ref().unwrap().ias_key,
        "ias_key_AAAABBBBCCCCDDDDEEEEFFFF"
    );
    assert_eq!(
        config.ias.as_ref().unwrap().ias_spid,
        "ias_spid_AAAABBBBCCCCDDDDEEEEFFF"
    );

    assert_eq!(
        config.audit.auditor_signatures_bytes.as_ref().unwrap()[0],
        b"godzilla.sign.sha256"
    )
}

fn runtime_config_with_env_vars() {
    env::set_var("IAS_KEY", "12345678901234567890123456789012");
    env::set_var("IAS_SPID", "90123456789012345678901234567890");
    let config =
        teaclave_config::RuntimeConfig::from_toml("./fixtures/runtime.config.toml").unwrap();
    assert_eq!(
        config.ias.as_ref().unwrap().ias_key,
        "12345678901234567890123456789012"
    );
    assert_eq!(
        config.ias.as_ref().unwrap().ias_spid,
        "90123456789012345678901234567890"
    );
}
