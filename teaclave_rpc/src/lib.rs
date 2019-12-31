use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait Service<V, U>
where
    U: Serialize + std::fmt::Debug,
    V: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    fn handle_request(&self, request: V) -> Result<U>;
}

pub mod channel;
mod protocol;
pub mod server;
mod transport;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::*;
    use crate::server::*;
    use anyhow::anyhow;
    use rustls::internal::pemfile;
    use std::fs;
    use std::io;
    use std::net;
    use std::net::TcpListener;
    use std::sync;
    use std::sync::mpsc;
    use std::thread;
    use webpki;

    #[derive(Serialize, Deserialize, Debug)]
    enum EchoRequest {
        Say(SayRequest),
        Sing,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct SayRequest {
        message: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    enum EchoResponse {
        Say(SayResponse),
        Sing,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct SayResponse {
        message: String,
    }
    struct EchoService;

    impl Service<EchoRequest, EchoResponse> for EchoService {
        fn handle_request(&self, request: EchoRequest) -> Result<EchoResponse> {
            println!("handle request: {:?}", request);
            let message = match request {
                EchoRequest::Say(s) => s.message,
                _ => return Err(anyhow!("error")),
            };
            Ok(EchoResponse::Say(SayResponse { message }))
        }
    }

    #[test]
    fn test_mpsc_echo() {
        struct EchoClient {
            channel: MpscChannel<EchoRequest, EchoResponse>,
        }

        impl EchoClient {
            fn new(channel: MpscChannel<EchoRequest, EchoResponse>) -> Self {
                Self { channel }
            }

            fn say(&mut self, req: SayRequest) -> Result<SayResponse> {
                let input = EchoRequest::Say(req);
                match self.channel.invoke(input)? {
                    EchoResponse::Say(r) => Ok(r),
                    _ => return Err(anyhow!("error")),
                }
            }
        }

        let service = EchoService;
        let (request_sender, request_receiver) = mpsc::channel::<EchoRequest>();
        let (response_sender, response_receiver) = mpsc::channel::<EchoResponse>();
        let mut server = MpscChannelServer::new(response_sender, request_receiver);
        thread::spawn(move || {
            server.start(service).unwrap();
        });

        std::thread::sleep(std::time::Duration::from_secs(1));
        let channel = MpscChannel::new(request_sender, response_receiver);
        let mut client = EchoClient::new(channel);
        let message = "Hello, World!".to_string();
        let request = SayRequest { message };

        let response = client.say(request);
        println!("{:?}", response);
    }

    #[test]
    fn test_trustedtls_echo() {
        struct EchoClient {
            channel: SgxTrustedTlsChannel<EchoRequest, EchoResponse>,
        }

        impl EchoClient {
            fn new(channel: SgxTrustedTlsChannel<EchoRequest, EchoResponse>) -> Self {
                Self { channel }
            }

            fn say(&mut self, req: SayRequest) -> Result<SayResponse> {
                let input = EchoRequest::Say(req);
                match self.channel.invoke(input)? {
                    EchoResponse::Say(r) => Ok(r),
                    _ => Err(anyhow!("error")),
                }
            }
        }

        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:12345").unwrap();
            for stream in listener.incoming() {
                let rc_config = get_server_config();
                let server_session = rustls::ServerSession::new(&rc_config);
                let stream = rustls::StreamOwned::new(server_session, stream.unwrap());
                let mut server = SgxTrustedTlsServer::<EchoResponse, EchoRequest>::new(stream);
                server.start(EchoService).unwrap();
            }
        });

        std::thread::sleep(std::time::Duration::from_secs(1));

        let stream = net::TcpStream::connect("127.0.0.1:12345").unwrap();

        let rc_config = get_client_config();
        let localhost = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
        let client_session = rustls::ClientSession::new(&rc_config, localhost);
        let stream = rustls::StreamOwned::new(client_session, stream);
        let client_channel = SgxTrustedTlsChannel::<EchoRequest, EchoResponse>::new(stream);

        let mut client = EchoClient::new(client_channel);
        let message = "Hello, World!".to_string();
        let request = SayRequest { message };

        let response = client.say(request);
        println!("{:?}", response);
    }

    fn get_client_config() -> sync::Arc<rustls::ClientConfig> {
        let mut client_config = rustls::ClientConfig::new();
        let mut rootbuf = io::BufReader::new(fs::File::open("test_ca/ca.cert").unwrap());
        client_config.root_store.add_pem_file(&mut rootbuf).unwrap();
        let rc_config = sync::Arc::new(client_config);
        rc_config
    }

    fn get_server_config() -> sync::Arc<rustls::ServerConfig> {
        let cert = pemfile::certs(&mut io::BufReader::new(
            fs::File::open("test_ca/end.fullchain").unwrap(),
        ))
        .unwrap();
        let private_key = &pemfile::pkcs8_private_keys(&mut io::BufReader::new(
            fs::File::open("test_ca/end.key").unwrap(),
        ))
        .unwrap()[0];

        let mut server_config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        server_config
            .set_single_cert(cert.clone(), private_key.clone())
            .unwrap();
        let rc_config = sync::Arc::new(server_config);
        rc_config
    }
}
