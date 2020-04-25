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
