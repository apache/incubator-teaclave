cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        pub use sgx_trusted_tls::SgxTrustedTlsServer;
    } else {
        pub use mpsc_channel::MpscChannelServer;
    }
}

#[cfg(feature = "mesalock_sgx")]
mod sgx_trusted_tls {
    use crate::config::SgxTrustedTlsServerConfig;
    use crate::transport::{ServerTransport, SgxTrustedTlsTransport};
    use crate::TeaclaveService;
    use anyhow::Result;
    use log::debug;
    use serde::{Deserialize, Serialize};

    pub struct SgxTrustedTlsServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        listener: std::net::TcpListener,
        tls_config: std::sync::Arc<rustls::ServerConfig>,
        maker: std::marker::PhantomData<(U, V)>,
    }

    impl<U, V> SgxTrustedTlsServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        pub fn new(
            listener: std::net::TcpListener,
            server_config: &SgxTrustedTlsServerConfig,
        ) -> SgxTrustedTlsServer<U, V> {
            Self {
                listener,
                tls_config: server_config.config.clone(),
                maker: std::marker::PhantomData::<(U, V)>,
            }
        }

        pub fn start<X>(&mut self, service: X) -> Result<()>
        where
            X: 'static
                + TeaclaveService<V, U>
                + Clone
                + Copy
                + core::marker::Send
                + core::marker::Sync,
        {
            let n_workers = 10;
            let pool = threadpool::ThreadPool::new(n_workers);
            for stream in self.listener.incoming() {
                let session = rustls::ServerSession::new(&self.tls_config);
                let tls_stream = rustls::StreamOwned::new(session, stream.unwrap());
                let mut transport = SgxTrustedTlsTransport::new(tls_stream);
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
}

#[cfg(not(feature = "mesalock_sgx"))]
mod mpsc_channel {
    use std::sync::mpsc;
    pub struct MpscChannelServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        transport: MpscChannelTransport<U, V>,
    }

    impl<U, V> MpscChannelServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        pub fn new(rx: mpsc::Sender<U>, tx: mpsc::Receiver<V>) -> Self {
            let transport = MpscChannelTransport { rx, tx };
            Self { transport }
        }

        pub fn start<X: TeaclaveService<V, U>>(&mut self, service: X) -> Result<()> {
            self.transport.serve(service)
        }
    }
}
