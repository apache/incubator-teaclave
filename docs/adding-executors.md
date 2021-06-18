---
permalink: /docs/adding-executors
---

# Adding Executors

Teaclave supports several function executors currently: `builtin`, `mesapy` and
`wamr` (WebAssembly Micro Runtime). For more information about current function
executors, please check [this link](../executor/README.md).

However, sometimes platform providers may found current executors built in
Teaclave are not applicable for hosting some services, and they want to use
their own executor or an executor shipped by the third-party to execute their
code (for example, written languages other than Python or WASM). They can then
modify Teaclave's source code to add a customized executor to run their
functions.

## Steps of Adding a New Executor

Executors can be either linked to Teaclave as a third-party library (e.g. Mesapy
executor) or built in Teaclave itself (e.g. builtin executor). The source code
of either type is located at `executor/src/`. The general steps for adding a
custom executor can be summarized in the following steps:

1. Create a public executor unit struct and implement the `TeaclaveExecutor` trait.
2. Re-export your new executor in `executor/src/lib.rs` to make it callable.
3. Add enums in `ExecutorType` and `Executor` (in `types/src/worker.rs`), as
   well as corresponding logics for handling the added enums.
4. Import and register the added executor in `worker/src/worker.rs`.
5. (Optional) Add unit test to your customized executor

### Linking Related Libraries

If the custom executor is embedded or ported to Teaclave, `extern` functions
might be introduced into Teaclave and thus you also need to tell the build
system where to find the library containing these external functions. You may
add this library in the linking command at script
`cmake/scripts/sgx_link_sign.sh`. The linker will try to find the library in
`${TEACLAVE_OUT_DIR}`, which is be parsed to `build/intermediate` in build
phase. Besides, you can also add several lines to generate or download the
library in `CMakeList.txt`. 

## Invoking the New Executor

Just call the API and remember to set `executor_type` to your new executor
type's name (the string used in `ExecutorType::try_from` match case) when
calling `register_function`, and set the `executor` to the executor's name
correspondingly.
