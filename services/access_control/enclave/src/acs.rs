// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

const MODEL_TEXT: &str = include_str!("../../model.conf");
extern "C" {
    fn acs_setup_model(model_text: *const c_char) -> i32;
    fn acs_enforce_request(request_type: *const c_char, request_content: *const c_char) -> i32;
}
#[cfg(test_mode)]
extern "C" {
    fn acs_announce_fact(fact_type: *const c_char, fact_vals: *const c_char) -> i32;
}

pub(crate) enum EnforceRequest {
    // user_access_data = usr, data
    UserAccessData(String, String),
    // user_access_function = usr, function
    UserAccessFunction(String, String),
    // user_access_task= usr, task
    UserAccessTask(String, String),
    // task_access_function = task, function
    TaskAccessFunction(String, String),
    // task_access_data = task, data
    TaskAccessData(String, String),
}

#[cfg(test_mode)]
pub(crate) enum AccessControlTerms {
    // data_owner = data, usr
    DataOwner(String, String),
    // function_owner = functoin, usr
    FunctionOwner(String, String),
    // is_public_function = function
    IsPublicFunction(String),
    // task_participant = task, usr
    TaskParticipant(String, String),
}

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

#[cfg(test_mode)]
pub(crate) fn init_mock_data() -> Result<()> {
    // mock data for AuthorizeData
    let term = AccessControlTerms::DataOwner("mock_data".to_string(), "mock_user_a".to_string());
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner("mock_data".to_string(), "mock_user_b".to_string());
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner("mock_data".to_string(), "mock_user_c".to_string());
    announce_fact(term)?;

    // mock data for AuthorizeFunction
    let term = AccessControlTerms::FunctionOwner(
        "mock_private_function".to_string(),
        "mock_private_function_owner".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::FunctionOwner(
        "mock_public_function".to_string(),
        "mock_public_function_owner".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::IsPublicFunction("mock_public_function".to_string());
    announce_fact(term)?;

    // mock data for AuthorizeTask
    let term = AccessControlTerms::TaskParticipant(
        "mock_task".to_string(),
        "mock_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::TaskParticipant(
        "mock_task".to_string(),
        "mock_participant_b".to_string(),
    );
    announce_fact(term)?;

    // mock data for AuthorizeStagedTask
    let term = AccessControlTerms::TaskParticipant(
        "mock_staged_task".to_string(),
        "mock_staged_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::TaskParticipant(
        "mock_staged_task".to_string(),
        "mock_staged_participant_b".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::FunctionOwner(
        "mock_staged_allowed_private_function".to_string(),
        "mock_staged_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::FunctionOwner(
        "mock_staged_disallowed_private_function".to_string(),
        "mock_staged_non_participant".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::IsPublicFunction("mock_staged_public_function".to_string());
    announce_fact(term)?;

    let term = AccessControlTerms::DataOwner(
        "mock_staged_allowed_data1".to_string(),
        "mock_staged_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner(
        "mock_staged_allowed_data2".to_string(),
        "mock_staged_participant_b".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner(
        "mock_staged_allowed_data3".to_string(),
        "mock_staged_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner(
        "mock_staged_allowed_data3".to_string(),
        "mock_staged_participant_b".to_string(),
    );
    announce_fact(term)?;

    let term = AccessControlTerms::DataOwner(
        "mock_staged_disallowed_data1".to_string(),
        "mock_staged_non_participant".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner(
        "mock_staged_disallowed_data2".to_string(),
        "mock_staged_participant_a".to_string(),
    );
    announce_fact(term)?;
    let term = AccessControlTerms::DataOwner(
        "mock_staged_disallowed_data2".to_string(),
        "mock_staged_non_participant".to_string(),
    );
    announce_fact(term)?;

    Ok(())
}

#[derive(Clone)]
pub(crate) struct AccessControlModule {
    lock: Arc<Mutex<u32>>,
}

impl AccessControlModule {
    pub(crate) fn new() -> Self {
        AccessControlModule {
            lock: Arc::new(Mutex::new(0)),
        }
    }

    pub(crate) fn enforce_request(&self, request: EnforceRequest) -> Result<bool> {
        let (request_type, request_content) = match request {
            EnforceRequest::UserAccessData(usr, data) => {
                let mut buffer = String::new();
                (usr, data).marshal(&mut buffer);
                ("user_access_data", buffer)
            }
            EnforceRequest::UserAccessFunction(usr, function) => {
                let mut buffer = String::new();
                (usr, function).marshal(&mut buffer);
                ("user_access_function", buffer)
            }
            EnforceRequest::UserAccessTask(usr, task) => {
                let mut buffer = String::new();
                (usr, task).marshal(&mut buffer);
                ("user_access_task", buffer)
            }
            EnforceRequest::TaskAccessFunction(task, function) => {
                let mut buffer = String::new();
                (task, function).marshal(&mut buffer);
                ("task_access_function", buffer)
            }
            EnforceRequest::TaskAccessData(task, data) => {
                let mut buffer = String::new();
                (task, data).marshal(&mut buffer);
                ("task_access_data", buffer)
            }
        };

        let c_request_type = CString::new(request_type.to_string())?;
        let c_request_content = CString::new(request_content)?;
        let _lock = self
            .lock
            .lock()
            .map_err(|_| anyhow!("failed to accquire lock"))?;
        let py_ret =
            unsafe { acs_enforce_request(c_request_type.as_ptr(), c_request_content.as_ptr()) };

        match py_ret {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(anyhow!("mesapy error")),
        }
    }
}
pub(crate) fn init_acs() -> Result<()> {
    let model_conf = CString::new(MODEL_TEXT).unwrap();
    let ec = unsafe { acs_setup_model(model_conf.as_ptr()) };

    if ec != 0 {
        Err(anyhow!("failed to init mesapy"))
    } else {
        #[cfg(test_mode)]
        init_mock_data()?;
        Ok(())
    }
}

#[cfg(test_mode)]
fn announce_fact(term: AccessControlTerms) -> Result<()> {
    let (term_type, term_fact) = match term {
        AccessControlTerms::DataOwner(data, usr) => {
            let mut buffer = String::new();
            (data, usr).marshal(&mut buffer);
            ("data_owner", buffer)
        }
        AccessControlTerms::FunctionOwner(function, usr) => {
            let mut buffer = String::new();
            (function, usr).marshal(&mut buffer);
            ("function_owner", buffer)
        }
        AccessControlTerms::IsPublicFunction(function) => {
            let mut buffer = String::new();
            (function,).marshal(&mut buffer);
            ("is_public_function", buffer)
        }
        AccessControlTerms::TaskParticipant(task, usr) => {
            let mut buffer = String::new();
            (task, usr).marshal(&mut buffer);
            ("task_participant", buffer)
        }
    };
    let c_term_type = CString::new(term_type.to_string())?;
    let c_term_fact = CString::new(term_fact)?;

    let py_ret = unsafe { acs_announce_fact(c_term_type.as_ptr(), c_term_fact.as_ptr()) };

    if py_ret != 0 {
        Err(anyhow!("mesapy error"))
    } else {
        Ok(())
    }
}
