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

use std::io::BufReader;
use std::io::Result;
use std::io::{Read, Write};
use std::mem::transmute;

pub fn send_vec<T>(sock: &mut T, buf: &[u8]) -> Result<()>
where
    T: Write,
{
    let buf_len: u64 = buf.len() as u64;
    let lbuf: [u8; 8] = unsafe { transmute(buf_len.to_be()) };
    sock.write_all(&lbuf)?;
    sock.write_all(&buf)?;
    sock.flush()?;
    Ok(())
}

pub fn recv_vec<T>(sock: &mut T) -> Result<Vec<u8>>
where
    T: Read,
{
    let mut br = BufReader::new(sock);
    let mut lbuf: [u8; 8] = [0; 8];
    br.read_exact(&mut lbuf)?;
    let buf_len: u64 = u64::from_be(unsafe { transmute::<[u8; 8], u64>(lbuf) });
    let mut recv_buf: Vec<u8> = vec![0u8; buf_len as usize];
    br.read_exact(&mut recv_buf)?;
    Ok(recv_buf)
}
