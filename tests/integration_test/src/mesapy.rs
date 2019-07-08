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
use crate::common::check_single_task_response;
use crate::config::USER_ONE;
use log::trace;

pub fn test_mesapy() {
    trace!(">>>>> mesapy");
    let request = base64::encode(
        "
def entrypoint():
    print('Hello Python World!')
    ",
    );

    let function_name = "mesapy_from_buffer";
    let payload = Some(request.as_str());

    check_single_task_response(
        &USER_ONE,
        function_name,
        payload,
        "marshal.loads(b\"\\x4E\")",
    ); // 4E is the marshaled "None"
}
