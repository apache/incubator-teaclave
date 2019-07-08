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

use mesatee_core::rpc::EnclaveService;
use mesatee_core::Result;
use std::marker::PhantomData;

use fns_proto::{InvokeTaskRequest, InvokeTaskResponse};

use crate::sgx::Executor;
use crate::trait_defs::TaskExecutor;

pub trait HandleRequest {
    fn handle_request(&self) -> Result<InvokeTaskResponse>;
}

impl HandleRequest for InvokeTaskRequest {
    fn handle_request(&self) -> Result<InvokeTaskResponse> {
        let mut executor = Executor::init(&self)?;
        let result = executor.execute()?;
        executor.finalize()?;
        let response = InvokeTaskResponse::new(&result);
        Ok(response)
    }
}

pub struct FNSEnclave<S, T> {
    state: i32,
    x: PhantomData<S>,
    y: PhantomData<T>,
}

impl<S, T> Default for FNSEnclave<S, T> {
    fn default() -> Self {
        FNSEnclave {
            state: 0,
            x: PhantomData::<S>,
            y: PhantomData::<T>,
        }
    }
}

impl EnclaveService<InvokeTaskRequest, InvokeTaskResponse>
    for FNSEnclave<InvokeTaskRequest, InvokeTaskResponse>
{
    fn handle_invoke(&mut self, input: InvokeTaskRequest) -> Result<InvokeTaskResponse> {
        trace!("handle_invoke invoked!");
        trace!("incoming payload = {:?}", input);
        self.state += 1;
        let response = input.handle_request()?;
        trace!("{}th round complete!", self.state);
        Ok(response)
    }
}
