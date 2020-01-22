use crate::protocol;
use crate::TeaclaveService;
use anyhow::{bail, Result};
use log::debug;
use rustls;
use serde::{Deserialize, Serialize};

pub(crate) trait ClientTransport {
    fn send<U, V>(&mut self, request: U) -> Result<V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug;
}

pub(crate) trait ServerTransport {
    fn serve<U, V, X>(&mut self, service: X) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
        X: TeaclaveService<V, U>;
}
pub(crate) struct SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    stream: rustls::StreamOwned<S, std::net::TcpStream>,
}

impl<S> SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    pub fn new(stream: rustls::StreamOwned<S, std::net::TcpStream>) -> SgxTrustedTlsTransport<S> {
        SgxTrustedTlsTransport::<S> { stream }
    }
}

impl<S> ClientTransport for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn send<U, V>(&mut self, request: U) -> Result<V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut protocol = protocol::JsonProtocol::new(&mut self.stream);
        protocol.write_message(request)?;
        protocol
            .read_message()
            .map_err(|_| anyhow::anyhow!("InternalError"))
    }
}

impl<S> ServerTransport for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn serve<U, V, X>(&mut self, service: X) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
        X: TeaclaveService<V, U>,
    {
        let mut protocol = protocol::JsonProtocol::new(&mut self.stream);

        loop {
            let request: V = match protocol.read_message() {
                Ok(r) => r,
                Err(e) => match e {
                    protocol::ProtocolError::IoError(_) => {
                        debug!("Connection disconnected.");
                        return Ok(());
                    }
                    _ => bail!("InternalError"),
                },
            };
            let response = service.handle_request(request);
            protocol.write_message(response)?;
        }
    }
}
