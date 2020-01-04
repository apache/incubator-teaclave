use crate::TeaclaveService;
use anyhow::Result;
use cfg_if::cfg_if;
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

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        pub(crate) use sgx_trusted_tls::SgxTrustedTlsTransport;
    } else {
        pub(crate) use mpsc_channel::MpscChannelTransport;
    }
}

#[cfg(feature = "mesalock_sgx")]
mod sgx_trusted_tls {
    use crate::protocol;
    use crate::transport::{ClientTransport, ServerTransport};
    use crate::TeaclaveService;
    use anyhow::Result;
    use log::debug;
    use rustls;
    use serde::{Deserialize, Serialize};

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
            SgxTrustedTlsTransport::<rustls::ClientSession> { stream }
        }

        pub fn new_server_with_stream(
            stream: rustls::StreamOwned<rustls::ServerSession, std::net::TcpStream>,
        ) -> SgxTrustedTlsTransport<rustls::ServerSession> {
            SgxTrustedTlsTransport::<rustls::ServerSession> { stream }
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
            X: TeaclaveService<V, U>,
        {
            let mut protocol = protocol::JsonProtocol::new(self);

            loop {
                debug!("read_message");
                let request: V = protocol.read_message()?;
                debug!("request: {:?}", request);
                let response: U = service.handle_request(request)?;
                debug!("response: {:?}", response);
                protocol.write_message(response)?;
                debug!("write_messagge done");
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
}

#[cfg(not(feature = "mesalock_sgx"))]
mod mpsc_channel {
    use std::sync::mpsc;

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
}
