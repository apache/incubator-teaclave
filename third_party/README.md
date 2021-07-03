---
permalink: /docs/codebase/third-party
---

# Third-Party Dependency Vendoring

For Teaclave, since all SGX/enclave dependencies are part of trusted computing base (TCB),
in order to ease auditing, ensure product stability, as well as reduce the
possibility of the [supply chain attack](https://en.wikipedia.org/wiki/Supply_chain_attack),
we vendored all TEE dependencies here. During the build process, both the
untrusted (i.e., the app part) and trusted components (i.e., the enclave part)
will only consume packages from this designated repository and will not
download any code from external package registry such as
[crates.io](https://crates.io).

Basically, we have these submodules:
  - `crates-io`: Rust crates vendored from crates.io for non-SGX components.
  - `crates-sgx`: Vendored crates ported for SGX (i.e., using Teaclave SGX SDK).
  - `mesapy`: a Python interpreter targeting SGX.
  - `rust-sgx-sdk`: Teaclave SGX SDK for standard libraries and Rust bindings of
    SGX libraries.
  - `webassembly-micro-runtime`: A sandboxed runtime to execute(interpret)
    WebAssembly bytecode.
