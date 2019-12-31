use crate::transport::*;
use crate::Service;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

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

    // fn new_with_client_attr(fd: c_int, client_attr: EnclaveAttr) -> Self {
    //     unimplemented!()
    // }

    pub fn start<X: Service<V, U>>(&mut self, service: X) -> Result<()> {
        self.transport.serve(service)
    }
}

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

    pub fn start<X: Service<V, U>>(&mut self, service: X) -> Result<()> {
        self.transport.serve(service)
    }
}
