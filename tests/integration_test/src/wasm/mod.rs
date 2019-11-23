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
use crate::common::launch_single_task;
use crate::config::USER_ONE;
use log::trace;

mod wasmi_basic;
mod wasmi_faas;

use wasmi_basic::BoundaryValue;
use wasmi_basic::FaasInterpreterError;

pub fn test_one_wasmi(wast_file: &str) {
    trace!(">>>>> test_one_wasmi");

    let commands = wasmi_faas::parse_a_wast(wast_file).unwrap();
    let actions = wasmi_faas::get_sgx_action(&commands);
    let request = serde_json::to_string(&actions).unwrap();

    let function_name = "wasmi_from_buffer";
    let payload = Some(request.as_str());
    let response = launch_single_task(&USER_ONE, function_name, payload);

    let results: Vec<Result<Option<BoundaryValue>, FaasInterpreterError>> =
        serde_json::from_str(&response).unwrap();
    assert!(wasmi_faas::match_result(commands, results));
}

pub fn test_wasmi() {
    let wast_list = vec![
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/int_exprs.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/conversions.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/nop.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/float_memory.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/call.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/memory.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/utf8-import-module.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/labels.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/align.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/memory_trap.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/br.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/globals.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/comments.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/get_local.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/float_literals.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/elem.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f64_bitwise.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/custom_section.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/inline-module.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/call_indirect.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/break-drop.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/unreached-invalid.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/utf8-import-field.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/loop.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/br_if.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/select.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/unwind.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/binary.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/tee_local.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/custom.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/start.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/float_misc.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/stack.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f32_cmp.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/i64.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/const.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/unreachable.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/switch.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/resizing.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/i32.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f64_cmp.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/int_literals.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/br_table.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/traps.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/return.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f64.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/type.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/fac.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/set_local.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/func.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f32.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/f32_bitwise.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/float_exprs.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/linking.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/skip-stack-guard-page.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/names.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/address.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/memory_redundancy.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/block.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/utf8-invalid-encoding.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/left-to-right.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/forward.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/typecheck.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/store_retval.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/imports.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/exports.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/endianness.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/func_ptrs.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/if.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/token.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/data.wast",
        "../../third_party/rust-sgx-sdk/samplecode/wasmi/test_input/utf8-custom-section-id.wast",
    ];
    for wfile in wast_list {
        test_one_wasmi(wfile);
    }
}
