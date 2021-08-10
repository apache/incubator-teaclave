#
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.
#
import os
import sgx_cffi
import _cffi_backend as backend

ffi = sgx_cffi.FFI(backend)

ffi.embedding_api("int acs_setup_model(const char *configuration);")
ffi.embedding_api("""int acs_enforce_request(const char *request_type,
                                             const char *request_content);""")
ffi.embedding_api("""int acs_announce_fact(const char *term_type,
                                           const char *term_fact);""")
with open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "acs_engine.py")) as f:
    ffi.embedding_init_code(f.read())
ffi.set_source('acs_py_enclave', '')
ffi.emit_c_code(os.environ.get('PYPY_FFI_OUTDIR', ".") + "/acs_py_enclave.c")
