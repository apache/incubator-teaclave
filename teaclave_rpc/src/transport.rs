use crate::protocol;
use crate::Service;
use anyhow::Result;
use rustls;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

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
        X: Service<V, U>;
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
    pub fn new_client_with_stream(
        stream: rustls::StreamOwned<rustls::ClientSession, std::net::TcpStream>,
    ) -> SgxTrustedTlsTransport<rustls::ClientSession> {
        SgxTrustedTlsTransport::<rustls::ClientSession> { stream: stream }
    }

    pub fn new_server_with_stream(
        stream: rustls::StreamOwned<rustls::ServerSession, std::net::TcpStream>,
    ) -> SgxTrustedTlsTransport<rustls::ServerSession> {
        SgxTrustedTlsTransport::<rustls::ServerSession> { stream: stream }
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
        let mut protocol = protocol::JsonProtocol::new(self);
        protocol.write_message(request)?;
        protocol.read_message()
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
        X: Service<V, U>,
    {
        loop {
            let mut protocol = protocol::JsonProtocol::new(self);
            let request: V = protocol.read_message()?;
            let response: U = service.handle_request(request)?;
            protocol.write_message(response)?;
        }
    }
}

impl<S> std::io::Read for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<S> std::io::Write for SgxTrustedTlsTransport<S>
where
    S: rustls::Session,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

pub(crate) struct MpscChannelTransport<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    pub rx: mpsc::Sender<U>,
    pub tx: mpsc::Receiver<V>,
}

impl<U, V> MpscChannelTransport<U, V>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    pub fn send(&mut self, request: U) -> Result<V> {
        self.rx.send(request).unwrap();
        Ok(self.tx.recv().unwrap())
    }

    pub fn serve<X>(&mut self, service: X) -> Result<()>
    where
        X: Service<V, U>,
    {
        loop {
            let request: V = self.tx.recv().unwrap();
            println!("recv request: {:?}", request);
            let response = service.handle_request(request).unwrap();
            println!("send response: {:?}", response);
            self.rx.send(response).unwrap();
        }
    }
}
