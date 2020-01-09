cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        pub use sgx_trusted_tls::SgxTrustedTlsChannel;
    } else {
        pub use mpsc_channel::MpscChannel;
    }
}

#[cfg(feature = "mesalock_sgx")]
mod sgx_trusted_tls {
    use crate::config::SgxTrustedTlsClientConfig;
    use crate::transport::{ClientTransport, SgxTrustedTlsTransport};
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    pub struct SgxTrustedTlsChannel<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        transport: SgxTrustedTlsTransport<rustls::ClientSession>,
        maker: std::marker::PhantomData<(U, V)>,
    }

    impl<U, V> SgxTrustedTlsChannel<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        pub fn new<A: std::net::ToSocketAddrs>(
            addr: A,
            hostname: &str,
            client_config: &SgxTrustedTlsClientConfig,
        ) -> Result<SgxTrustedTlsChannel<U, V>> {
            let stream = std::net::TcpStream::connect(addr)?;
            let hostname = webpki::DNSNameRef::try_from_ascii_str(hostname)?;
            let session = rustls::ClientSession::new(&client_config.config, hostname);
            let tls_stream = rustls::StreamOwned::new(session, stream);
            let transport = SgxTrustedTlsTransport::new(tls_stream);

            Ok(Self {
                transport,
                maker: std::marker::PhantomData::<(U, V)>,
            })
        }

        pub fn invoke(&mut self, input: U) -> Result<V> {
            self.transport.send(input)
        }
    }
}

#[cfg(not(feature = "mesalock_sgx"))]
mod mpsc_channel {
    use std::sync::mpsc;

    pub struct MpscChannel<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        transport: MpscChannelTransport<U, V>,
    }

    impl<U, V> MpscChannel<U, V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        pub fn new(rx: mpsc::Sender<U>, tx: mpsc::Receiver<V>) -> Self {
            let transport = MpscChannelTransport { rx, tx };
            Self { transport }
        }

        pub fn invoke(&mut self, input: U) -> Result<V> {
            self.transport.send(input)
        }
    }
}
