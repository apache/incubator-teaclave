use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io;
use std::mem::transmute;

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

    pub fn read_message<V>(&mut self) -> Result<V>
    where
        V: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let mut header = [0u8; 8];

        self.transport.read_exact(&mut header)?;
        let buf_len = u64::from_be(unsafe { transmute::<[u8; 8], u64>(header) });

        let mut recv_buf: Vec<u8> = vec![0u8; buf_len as usize];
        self.transport.read_exact(&mut recv_buf)?;

        let r: V = serde_json::from_slice(&recv_buf)?;

        Ok(r)
    }

    pub fn write_message<U>(&mut self, message: U) -> Result<()>
    where
        U: Serialize + std::fmt::Debug,
    {
        let message_vec = serde_json::to_vec(&message)?;

        let buf_len = message_vec.len() as u64;
        let header = unsafe { transmute::<u64, [u8; 8]>(buf_len.to_be()) };

        self.transport.write(&header)?;
        self.transport.write_all(&message_vec)?;
        self.transport.flush()?;

        Ok(())
    }
}
