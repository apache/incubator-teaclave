// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate log;

use mesatee_core::prelude::*;
use mesatee_core::{config, Result};

use std::net::TcpListener;
use std::os::unix::io::IntoRawFd;
use threadpool::ThreadPool;

use mesatee_binder::TeeBinder;
use std::sync::Arc;

fn main() -> Result<()> {
    env_logger::init();

    let tee = match TeeBinder::new("acs", 1) {
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
            let _ = ref_tee.finalize();
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    run_access_control_service(tee)?;

    Ok(())
}

fn run_access_control_service(tee: Arc<TeeBinder>) -> Result<()> {
    info!("Running as ACS Server ...");

    let config = config::Internal::acs();
    let listener = TcpListener::bind(config.addr)?;
    let port = config.addr.port();

    let n_workers = 10;
    let pool = ThreadPool::new(n_workers);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tee = tee.clone();
                pool.execute(move || {
                    debug!("new worker from {:?}", stream.peer_addr());
                    let fd = stream.into_raw_fd();
                    let input = ServeConnectionInput::new(fd, port);
                    let cmd = ECallCommand::ServeConnection;
                    let _ = tee
                        .invoke::<ServeConnectionInput, ServeConnectionOutput>(cmd.into(), input);
                });
            }
            Err(e) => warn!("couldn't get client: {:?}", e),
        }
    }

    Ok(())
}
