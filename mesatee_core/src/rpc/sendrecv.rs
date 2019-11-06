// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::io::BufReader;
use std::io::{Read, Write};
use std::mem::transmute;

use crate::{Error, ErrorKind, Result};

// TODO: configuration in config.toml
// message size over MAX_MSG_SIZE will cause RPC error
const MAX_MSG_SIZE: u64 = 0x64000;

fn get_send_vec(mut to_send: &mut Vec<u8>) -> Vec<u8> {
    let buf_len: u64 = to_send.len() as u64;
    let lbuf: [u8; 8] = unsafe { transmute(buf_len.to_be()) };
    let mut all_data: Vec<u8> = lbuf.to_vec();
    all_data.append(&mut to_send);

    all_data
}

pub fn send_vec<T>(sock: &mut T, mut buff: Vec<u8>) -> Result<()>
where
    T: Write,
{
    if buff.len() as u64 > MAX_MSG_SIZE {
        return Err(Error::from(ErrorKind::MsgSizeLimitExceedError));
    }
    let send_vec = get_send_vec(&mut buff);

    sock.write_all(&send_vec)?;
    sock.flush()?;

    Ok(())
}

pub fn receive_vec<T>(sock: &mut T) -> Result<Vec<u8>>
where
    T: Read,
{
    let mut br = BufReader::new(sock);
    let mut lbuf: [u8; 8] = [0; 8];

    br.read_exact(&mut lbuf)?;

    let buf_len: u64 = u64::from_be(unsafe { transmute::<[u8; 8], u64>(lbuf) });
    if buf_len > MAX_MSG_SIZE {
        return Err(Error::from(ErrorKind::MsgSizeLimitExceedError));
    }

    let mut recv_buf: Vec<u8> = vec![0u8; buf_len as usize];

    br.read_exact(&mut recv_buf)?;

    Ok(recv_buf)
}
