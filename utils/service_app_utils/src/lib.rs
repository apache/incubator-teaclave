use log::{error, info};
use std::sync::Arc;
use teaclave_binder::TeeBinder;

pub struct ServiceEnclaveBuilder;

impl ServiceEnclaveBuilder {
    pub fn init_tee_binder(enclave_name: &str) -> anyhow::Result<std::sync::Arc<TeeBinder>> {
        env_logger::init();

        let tee = match TeeBinder::new(enclave_name, 1) {
            Ok(r) => {
                info!("Init TEE Successfully!");
                r
            }
            Err(x) => {
                error!("Init TEE Failed {}!", x);
                std::process::exit(-1)
            }
        };

        let tee = Arc::new(tee);

        {
            let ref_tee = tee.clone();
            ctrlc::set_handler(move || {
                info!("\nCTRL+C pressed. Destroying server enclave");
                ref_tee.finalize();
                std::process::exit(0);
            })
            .expect("Error setting Ctrl-C handler");
        }

        Ok(tee)
    }
}
