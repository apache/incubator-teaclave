use crate::protocol;
use crate::Request;
use crate::TeaclaveService;
use anyhow::{bail, Result};
use log::debug;
use rustls;
use serde::{Deserialize, Serialize};
use std::prelude::v1::*;

pub(crate) trait ClientTransport {
    fn send<U, V>(
        &mut self,
        request: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V>
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
    fn send<U, V>(
        &mut self,
        request: Request<U>,
    ) -> teaclave_types::TeaclaveServiceResponseResult<V>
    where
        U: Serialize + std::fmt::Debug,
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut protocol = protocol::JsonProtocol::new(&mut self.stream);
        protocol.write_message(request).map_err(|_| {
            teaclave_types::TeaclaveServiceResponseError::InternalError(
                "protocol error".to_string(),
            )
        })?;
        protocol.read_message::<protocol::JsonProtocolResult<
                V,
                teaclave_types::TeaclaveServiceResponseError,
            >>()
            .map_err(|_| teaclave_types::TeaclaveServiceResponseError::InternalError("protocol error".to_string()))?
            .into()
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
            let request: Request<V> = match protocol.read_message::<Request<V>>() {
                Ok(r) => r,
                Err(e) => match e {
                    protocol::ProtocolError::IoError(_) => {
                        debug!("Connection disconnected.");
                        return Ok(());
                    }
                    _ => bail!("InternalError"),
                },
            };
            let response: protocol::JsonProtocolResult<
                U,
                teaclave_types::TeaclaveServiceResponseError,
            > = service.handle_request(request).into();
            protocol.write_message(response)?;
        }
    }
}
