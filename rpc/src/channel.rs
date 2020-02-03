use crate::config::SgxTrustedTlsClientConfig;
use crate::transport::{ClientTransport, SgxTrustedTlsTransport};
use crate::Request;
use anyhow::anyhow;
use anyhow::Result;
use http::Uri;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub fn new(
        address: &str,
        client_config: &SgxTrustedTlsClientConfig,
    ) -> Result<SgxTrustedTlsChannel<U, V>> {
        let uri = address.parse::<Uri>()?;
        let hostname = uri.host().ok_or_else(|| anyhow!("Invalid hostname."))?;
        let stream = std::net::TcpStream::connect(address)?;
        let hostname = webpki::DNSNameRef::try_from_ascii_str(hostname)?;
        let session = rustls::ClientSession::new(&Arc::new(client_config.config.clone()), hostname);
        let tls_stream = rustls::StreamOwned::new(session, stream);
        let transport = SgxTrustedTlsTransport::new(tls_stream);

        Ok(Self {
            transport,
            maker: std::marker::PhantomData::<(U, V)>,
        })
    }

    pub fn invoke(
        &mut self,
        input: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V> {
        self.transport.send(input)
    }
}
