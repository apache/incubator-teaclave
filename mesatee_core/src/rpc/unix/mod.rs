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

//! Ding-Sun RPC (dsrpc) for MesaTEE-SGX
//!
//! MesaTEE-SGX uses direct ECALL/OCALL interfaces on local, and
//! socket for remote.

use serde::{de::DeserializeOwned, Serialize};
use sgx_types::c_int;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;

use crate::rpc::{EnclaveService, RpcClient, RpcServer};
#[cfg(not(feature = "mesalock_sgx"))]
use std::os::unix::io::FromRawFd;

use crate::Result;

// TODO: make the following comments into doc comments starting with `//!`.
// Sgx Socket RPC always depends on a socket which is always managed by fd.
// Before launching a Sgx Socket server, one should run a thread pool and
// properly run a "accept loop" like before. On accept, use `into_raw_fd` to
// pass the fd to the enclave.
//
// ```
//     println!("Running as server...");
//  let listener = TcpListener::bind("0.0.0.0:3443").unwrap();
//  match listener.accept() {
//      Ok((socket, addr)) => {
//          println!("new client from {:?}", addr);
//          let mut retval = sgx_status_t::SGX_SUCCESS;
//          let result = unsafe {
//              run_server(enclave.geteid(), &mut retval, socket.into_raw_fd(), sign_type)
//          };
//          match result {
//              sgx_status_t::SGX_SUCCESS => {
//                  println!("ECALL success!");
//              },
//              _ => {
//                  println!("[-] ECALL Enclave Failed {}!", result.as_str());
//                  return;
//              }
//          }
//      }
//      Err(e) => println!("couldn't get client: {:?}", e),
//  }
//```
// `into_raw_fd` consumes the stream. So socket would **not** be dropped on
// leaving its scope, and **not** triggers its dtor which closes the connection.
// In MesaTEE, the stream is re-constructed in an enclave by `from_raw_fd`. So
// the dtor is invoked when the in-enclave stream leaves its scope. Pairing
// the `into_raw_fd` and `from_raw_fd` is **required**.
pub struct PipeConfig {
    fd: c_int,
}

impl PipeConfig {
    pub fn new(fd: c_int) -> Self {
        PipeConfig { fd }
    }

    pub fn get(&self) -> c_int {
        self.fd
    }
}

pub type PipeClientConfig = PipeConfig;

// Pipe stores the `TcpStream` created from the fd comes from Config.
pub struct Pipe<U, V, X> {
    inner: TcpStream,
    u: PhantomData<U>,
    v: PhantomData<V>,
    x: PhantomData<X>,
}

impl<U, V, X> Read for Pipe<U, V, X> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<U, V, X> Write for Pipe<U, V, X> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<U, V, X> RpcServer<U, V, X> for Pipe<U, V, X>
where
    U: DeserializeOwned,
    V: Serialize,
    X: EnclaveService<U, V>,
{
    type Config = PipeConfig;
    // type X = Box<EnclaveService<U, V>>;
    // The `TcpStream::new()` function is different from Rust's design.
    // The SGX version takes a fd as a input and return an `Option`.
    fn start(config: Self::Config) -> Result<Self> {
        Ok(Pipe {
            #[cfg(feature = "mesalock_sgx")]
            inner: TcpStream::new(config.get())?,
            #[cfg(not(feature = "mesalock_sgx"))]
            inner: unsafe { TcpStream::from_raw_fd(config.get()) },
            u: PhantomData::<U>,
            v: PhantomData::<V>,
            x: PhantomData::<X>,
        })
    }

    // Use default impplementation
    // fn serve(&mut self, mut s: X) -> Result<()>;
}

pub struct PipeClient<U, V> {
    inner: TcpStream,
    u: PhantomData<U>,
    v: PhantomData<V>,
}

impl<U, V> Read for PipeClient<U, V> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<U, V> Write for PipeClient<U, V> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<U, V> RpcClient<U, V> for PipeClient<U, V>
where
    U: Serialize,
    V: DeserializeOwned,
{
    type Config = PipeClientConfig;

    fn open(config: Self::Config) -> Result<Self> {
        Ok(PipeClient {
            #[cfg(feature = "mesalock_sgx")]
            inner: TcpStream::new(config.get())?,
            #[cfg(not(feature = "mesalock_sgx"))]
            inner: unsafe { TcpStream::from_raw_fd(config.get()) },
            u: PhantomData::<U>,
            v: PhantomData::<V>,
        })
    }

    // Use default implementation
    // fn invoke(&mut self, input: U) -> Result<V>;
}
