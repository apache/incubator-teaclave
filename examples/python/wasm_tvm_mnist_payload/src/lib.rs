/*
* Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
*/

#[macro_use]
extern crate lazy_static;

mod types;
pub mod utils;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::{collections::HashMap, convert::TryFrom, env, sync::Mutex};
use tvm_graph_rt::{Graph, GraphExecutor, SystemLibModule, Tensor as TVMTensor};
use types::Tensor;

#[no_mangle]
pub extern "C" fn entrypoint(argc: c_int, argv: *const *const c_char) -> i32 {
    assert_eq!(argc, 2);

    // convert `argv` to `Vec<str>`
    let argv: Vec<_> = (0..argc)
        .map(|i| unsafe { CStr::from_ptr(*argv.add(i as usize)).to_string_lossy() })
        .collect();

    // Arguments are referenced in ODD indices
    run(argv[1].as_ref())
}

lazy_static! {
    static ref SYSLIB: SystemLibModule = SystemLibModule::default();
    static ref GRAPH_EXECUTOR: Mutex<GraphExecutor<'static, 'static>> = {
        let graph = Graph::try_from(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/outlib/graph.json"
        )))
        .unwrap();

        let params_bytes =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/outlib/graph.params"));
        let params = tvm_graph_rt::load_param_dict(params_bytes)
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, v.to_owned()))
            .collect::<HashMap<String, TVMTensor<'static>>>();

        let mut exec = GraphExecutor::new(graph, &*SYSLIB).unwrap();

        exec.load_params(params);

        Mutex::new(exec)
    };
}

#[no_mangle]
pub extern "C" fn run(input_file: &str) -> i32 {
    let in_tensor = utils::load_input(input_file);
    let input: TVMTensor = in_tensor.as_dltensor().into();

    // since this executor is not multi-threaded, we can acquire lock once
    let mut executor = GRAPH_EXECUTOR.lock().unwrap();

    executor.set_input("Input3", input);

    executor.run();

    let output = executor.get_output(0).unwrap().as_dltensor(false);

    let out_tensor: Tensor = output.into();
    utils::handle_output(out_tensor)
}
