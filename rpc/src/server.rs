use crate::config::SgxTrustedTlsServerConfig;
use crate::transport::{ServerTransport, SgxTrustedTlsTransport};
use crate::utils;
use crate::TeaclaveService;
use anyhow::Result;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};

pub struct SgxTrustedTlsServer<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    addr: std::net::SocketAddr,
    tls_config: SgxTrustedTlsServerConfig,
    tcp_nodelay: bool,
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
            maker: std::marker::PhantomData::<(U, V)>,
        }
    }

    pub fn tcp_nodelay(self, enabled: bool) -> Self {
        Self {
            tcp_nodelay: enabled,
            ..self
        }
    }

    pub fn start<X>(&mut self, service: X) -> Result<()>
    where
        X: 'static + TeaclaveService<V, U> + Clone + core::marker::Send,
    {
        let n_workers = utils::get_tcs_num();
        let pool = threadpool::ThreadPool::new(n_workers);
        let listener = std::net::TcpListener::bind(self.addr)?;
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
