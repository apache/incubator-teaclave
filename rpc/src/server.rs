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
    use serde::{Deserialize, Serialize};

    pub struct SgxTrustedTlsServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        transport: SgxTrustedTlsTransport<rustls::ServerSession>,
        maker: std::marker::PhantomData<(U, V)>,
    }

    impl<U, V> SgxTrustedTlsServer<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        pub fn new(
            stream: rustls::StreamOwned<rustls::ServerSession, std::net::TcpStream>,
        ) -> SgxTrustedTlsServer<U, V> {
            let transport =
                SgxTrustedTlsTransport::<rustls::ServerSession>::new_server_with_stream(stream);
            SgxTrustedTlsServer {
                transport,
                maker: std::marker::PhantomData::<(U, V)>,
            }
        }

        pub fn new_with_config(
            stream: std::net::TcpStream,
            server_config: &SgxTrustedTlsServerConfig,
        ) -> SgxTrustedTlsServer<U, V> {
            let session = rustls::ServerSession::new(&server_config.config);
            let stream = rustls::StreamOwned::new(session, stream);
            Self::new(stream)
        }

        pub fn start<X: TeaclaveService<V, U>>(&mut self, service: X) -> Result<()> {
            self.transport.serve(service)
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
