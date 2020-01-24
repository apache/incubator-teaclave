use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use teaclave_binder::TeeBinder;

pub struct ServiceEnclaveBuilder;

impl ServiceEnclaveBuilder {
    pub fn init_tee_binder(enclave_name: &str) -> Result<TeeBinder> {
        env_logger::init();

        TeeBinder::new(enclave_name, 1)
    }
}
