use crate::transport::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

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
        stream: rustls::StreamOwned<rustls::ClientSession, std::net::TcpStream>,
    ) -> SgxTrustedTlsChannel<U, V> {
        let transport =
            SgxTrustedTlsTransport::<rustls::ServerSession>::new_client_with_stream(stream);
        Self {
            transport,
            maker: std::marker::PhantomData::<(U, V)>,
        }
    }

    pub fn invoke(&mut self, input: U) -> Result<V> {
        self.transport.send(input)
    }
}

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
