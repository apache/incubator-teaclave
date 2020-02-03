use crate::config::SgxTrustedTlsServerConfig;
use crate::transport::{ServerTransport, SgxTrustedTlsTransport};
use crate::utils;
use crate::TeaclaveService;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SgxTrustedTlsServer<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    addr: std::net::SocketAddr,
    tls_config: std::sync::Arc<rustls::ServerConfig>,
    maker: std::marker::PhantomData<(U, V)>,
}

impl<U, V> SgxTrustedTlsServer<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    pub fn new(
        addr: std::net::SocketAddr,
        server_config: &SgxTrustedTlsServerConfig,
    ) -> SgxTrustedTlsServer<U, V> {
        Self {
            addr,
            tls_config: Arc::new(server_config.config.clone()),
            maker: std::marker::PhantomData::<(U, V)>,
        }
    }

    pub fn start<X>(&mut self, service: X) -> Result<()>
    where
        X: 'static + TeaclaveService<V, U> + Clone + core::marker::Send,
    {
        let n_workers = utils::get_tcs_num();
        let pool = threadpool::ThreadPool::new(n_workers);
        let listener = std::net::TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            let session = rustls::ServerSession::new(&self.tls_config);
            let tls_stream = rustls::StreamOwned::new(session, stream.unwrap());
            let mut transport = SgxTrustedTlsTransport::new(tls_stream);
            let service = service.clone();
            pool.execute(move || match transport.serve(service) {
                Ok(_) => (),
                Err(e) => {
                    debug!("serve error: {:?}", e);
                }
            });
        }
        Ok(())
    }
}
