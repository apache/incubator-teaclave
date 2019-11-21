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
use mesatee_core::{ErrorKind, Result};
use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::c_char;

use acs_proto::*;

pub trait PyMarshallable {
    fn marshal(&self, buffer: &mut String);
}

impl<T> PyMarshallable for (T,)
where
    T: PyMarshallable,
{
    fn marshal(&self, buffer: &mut String) {
        buffer.push('[');
        self.0.marshal(buffer);
        buffer.push(']');
    }
}

impl<U, V> PyMarshallable for (U, V)
where
    U: PyMarshallable,
    V: PyMarshallable,
{
    fn marshal(&self, buffer: &mut String) {
        buffer.push('[');
        self.0.marshal(buffer);
        buffer.push(',');
        self.1.marshal(buffer);
        buffer.push(']');
    }
}

impl<X, Y, Z> PyMarshallable for (X, Y, Z)
where
    X: PyMarshallable,
    Y: PyMarshallable,
    Z: PyMarshallable,
{
    fn marshal(&self, buffer: &mut String) {
        buffer.push('[');
        self.0.marshal(buffer);
        buffer.push(',');
        self.1.marshal(buffer);
        buffer.push(',');
        self.2.marshal(buffer);
        buffer.push(']');
    }
}

impl<T> PyMarshallable for [T]
where
    T: PyMarshallable,
{
    fn marshal(&self, buffer: &mut String) {
        buffer.push('[');
        for t in self {
            t.marshal(buffer);
            buffer.push(',');
        }
        buffer.push(']');
    }
}

impl<T: PyMarshallable> PyMarshallable for &HashSet<T> {
    fn marshal(&self, buffer: &mut String) {
        buffer.push_str("set([");
        for t in *self {
            t.marshal(buffer);
            buffer.push(',');
        }
        buffer.push_str("])");
    }
}

impl PyMarshallable for String {
    fn marshal(&self, buffer: &mut String) {
        buffer.push('\'');
        buffer.push_str(self);
        buffer.push('\'');
    }
}

impl PyMarshallable for &String {
    fn marshal(&self, buffer: &mut String) {
        buffer.push('\'');
        buffer.push_str(self);
        buffer.push('\'');
    }
}

pub trait HandleRequest {
    fn handle_request(&self) -> Result<ACSResponse>;
}

extern "C" {
    fn acs_enforce_request(request_type: *const c_char, request_content: *const c_char) -> i32;
    fn acs_announce_fact(fact_type: *const c_char, fact_vals: *const c_char) -> i32;
}

impl HandleRequest for EnforceRequest {
    fn handle_request(&self) -> Result<ACSResponse> {
        let (request_type, request_content) = match self {
            EnforceRequest::LaunchTask(usr, task) => {
                let mut buffer = String::new();
                (usr, task).marshal(&mut buffer);
                ("launch_task", buffer)
            }
            EnforceRequest::AccessData(task, data) => {
                let mut buffer = String::new();
                (task, data).marshal(&mut buffer);
                ("access_data", buffer)
            }
            EnforceRequest::DeleteData(usr, data) => {
                let mut buffer = String::new();
                (usr, data).marshal(&mut buffer);
                ("delete_data", buffer)
            }
            EnforceRequest::AccessScript(task, script) => {
                let mut buffer = String::new();
                (task, script).marshal(&mut buffer);
                ("access_script", buffer)
            }
            EnforceRequest::DeleteScript(usr, script) => {
                let mut buffer = String::new();
                (usr, script).marshal(&mut buffer);
                ("delete_script", buffer)
            }
        };

        let c_request_type = CString::new(request_type.to_string()).unwrap();
        let c_request_content = CString::new(request_content).unwrap();
        let py_ret =
            unsafe { acs_enforce_request(c_request_type.as_ptr(), c_request_content.as_ptr()) };

        match py_ret {
            0 => Ok(ACSResponse::Enforce(false)),
            1 => Ok(ACSResponse::Enforce(true)),
            _ => Err(ErrorKind::MesaPyError.into()),
        }
    }
}

impl HandleRequest for AnnounceRequest {
    fn handle_request(&self) -> Result<ACSResponse> {
        for fact in &self.facts {
            use AccessControlTerms::*;
            let (term_type, term_fact) = match fact {
                TaskCreator(task, usr) => {
                    let mut buffer = String::new();
                    (task, usr).marshal(&mut buffer);
                    ("task_creator", buffer)
                }
                TaskParticipant(task, usr) => {
                    let mut buffer = String::new();
                    (task, usr).marshal(&mut buffer);
                    ("task_participant", buffer)
                }
                DataOwner(data, usr) => {
                    let mut buffer = String::new();
                    (data, usr).marshal(&mut buffer);
                    ("data_owner", buffer)
                }
                ScriptOwner(script, usr) => {
                    let mut buffer = String::new();
                    (script, usr).marshal(&mut buffer);
                    ("script_owner", buffer)
                }
                IsPublicScript(script) => {
                    let mut buffer = String::new();
                    (script,).marshal(&mut buffer);
                    ("is_public_script", buffer)
                }
            };

            let c_term_type = CString::new(term_type.to_string()).unwrap();
            let c_term_fact = CString::new(term_fact).unwrap();

            let py_ret = unsafe { acs_announce_fact(c_term_type.as_ptr(), c_term_fact.as_ptr()) };

            if py_ret != 0 {
                return Err(ErrorKind::MesaPyError.into());
            }
        }
        Ok(ACSResponse::Announce)
    }
}

pub struct ACSEnclave;

impl Default for ACSEnclave {
    fn default() -> Self {
        ACSEnclave {}
    }
}

impl EnclaveService<ACSRequest, ACSResponse> for ACSEnclave {
    fn handle_invoke(&mut self, input: ACSRequest) -> Result<ACSResponse> {
        debug!("handle_invoke invoked!");
        debug!("incoming payload = {:?}", input);

        let response = match input {
            ACSRequest::Enforce(req) => req.handle_request()?,
            ACSRequest::Announce(req) => req.handle_request()?,
        };

        Ok(response)
    }
}
