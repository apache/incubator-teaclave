// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::config::SgxTrustedTlsServerConfig;
use crate::transport::{ServerTransport, SgxTrustedTlsTransport};
use crate::TeaclaveService;
use anyhow::Result;
#[cfg(not(feature = "mesalock_sgx"))]
use libc::setsockopt;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
#[cfg(feature = "mesalock_sgx")]
use sgx_libc as libc;
#[cfg(feature = "mesalock_sgx")]
use sgx_libc::ocall::setsockopt;
use std::{io, mem, os::unix::io::AsRawFd};

pub struct SgxTrustedTlsServer<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    addr: std::net::SocketAddr,
    tls_config: SgxTrustedTlsServerConfig,
    tcp_nodelay: bool,
    n_workers: usize,
    maker: std::marker::PhantomData<(U, V)>,
}

impl<U, V> SgxTrustedTlsServer<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    pub fn new(
        addr: std::net::SocketAddr,
        server_config: SgxTrustedTlsServerConfig,
    ) -> SgxTrustedTlsServer<U, V> {
        Self {
            addr,
            tls_config: server_config,
            tcp_nodelay: true,
            n_workers: 8,
            maker: std::marker::PhantomData::<(U, V)>,
        }
    }

    pub fn tcp_nodelay(self, enabled: bool) -> Self {
        Self {
            tcp_nodelay: enabled,
            ..self
        }
    }

    pub fn n_workers(self, n: usize) -> Self {
        Self {
            n_workers: n,
            ..self
        }
    }

    pub fn start<X>(&mut self, service: X) -> Result<()>
    where
        X: 'static + TeaclaveService<V, U> + Clone + core::marker::Send,
    {
        let pool = threadpool::ThreadPool::new(self.n_workers);
        let listener = std::net::TcpListener::bind(self.addr)?;
        unsafe {
            let optval: libc::c_int = 1;
            let ret = setsockopt(
                listener.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_REUSEPORT,
                &optval as *const _ as *const libc::c_void,
                mem::size_of_val(&optval) as libc::socklen_t,
            );
            if ret != 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        let mut tls_config_ref = self.tls_config.server_config();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Before introducing async into enclave, we check
                    // freshness for every incoming connection.
                    if self.tls_config.need_refresh() {
                        debug!("Attestation report is outdated, need to refresh");
                        self.tls_config.refresh_server_config()?;
                        tls_config_ref = self.tls_config.server_config();
                    }

                    if let Err(e) = stream.set_nodelay(self.tcp_nodelay) {
                        warn!("Cannot set_nodelay: {:}", e);
                        continue;
                    }
                    let session = rustls::ServerSession::new(&tls_config_ref);
                    let tls_stream = rustls::StreamOwned::new(session, stream);
                    let mut transport = SgxTrustedTlsTransport::new(tls_stream);
                    let service = service.clone();
                    pool.execute(move || match transport.serve(service) {
                        Ok(_) => (),
                        Err(e) => {
                            debug!("serve error: {:?}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Incoming error: {:}", e);
                }
            }
        }
        Ok(())
    }
}
