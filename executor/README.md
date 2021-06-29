---
permalink: /docs/codebase/executor
---

# Function Executors

Function executor is one of the core component in a FaaS platform to provide
execution runtime for running user-defined functions. In Teaclave, we aim to
provide safe, secure and versatile function executors, which can guarantee the
confidentiality of security-sensitive data during computation, and also support
functions written in different languages. In addition, we are working hard to
achieve better security guarantees such as memory safety.

In Teaclave, there are three executors to native, Python, and WebAssembly functions.
- **Builtin Executor**: There are many useful built-in functions which are statically
  compiled with Teaclave. Normally, these built-in functions are implemented in
  Rust, and can provide better (native) performance. The Builtin executor is to
  dispatch function invocation requests to corresponding built-in function
  implementations.
- **MesaPy Executor**: The MesaPy executor provides a Python interpreter in SGX.
  User-defined Python functions can be executed in the MesaPy executor. The
  executor also provides interfaces to fetch and store data through the runtime.
- **WAMR Executor**: WebAssembly Micro Runtime (WAMR) is integrated into
  Teaclave to provide a interpreter for WebAssembly bytecode. Please refer to
  the [WebAssembly Executor Document](../docs/executing-wasm.md) for more
  details on its usage.

To add a new executor, you can implement the `TeaclaveExecutor` trait (basically
implement the `execute` function). Then, register the executor in the Teaclave
worker. At last, the execution service will dispatch functions to the specific
executor.
