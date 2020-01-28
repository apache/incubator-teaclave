use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::mem::transmute;
use std::vec::Vec;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IoError")]
    IoError(#[from] io::Error),
    #[error("SerdeError")]
    SerdeError(#[from] serde_json::error::Error),
}

pub(crate) struct JsonProtocol<'a, T>
where
    T: io::Read + io::Write,
{
    pub transport: &'a mut T,
}

impl<'a, T> JsonProtocol<'a, T>
where
    T: io::Read + io::Write,
{
    pub fn new(transport: &'a mut T) -> JsonProtocol<'a, T> {
        Self { transport }
    }

    pub fn read_message<V>(&mut self) -> std::result::Result<V, ProtocolError>
    where
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut header = [0u8; 8];

        self.transport.read_exact(&mut header)?;
        let buf_len = u64::from_be(unsafe { transmute::<[u8; 8], u64>(header) });

        let mut recv_buf: Vec<u8> = vec![0u8; buf_len as usize];
        self.transport.read_exact(&mut recv_buf)?;

        debug!("Recv: {}", std::string::String::from_utf8_lossy(&recv_buf));
        let r: V = serde_json::from_slice(&recv_buf)?;

        Ok(r)
    }

    pub fn write_message<U>(&mut self, message: U) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
    {
        let send_buf = serde_json::to_vec(&message)?;

        debug!("Send: {}", std::string::String::from_utf8_lossy(&send_buf));

        let buf_len = send_buf.len() as u64;
        let header = unsafe { transmute::<u64, [u8; 8]>(buf_len.to_be()) };

        self.transport.write(&header)?;
        self.transport.write_all(&send_buf)?;
        self.transport.flush()?;

        Ok(())
    }
}
