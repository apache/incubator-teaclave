---
permalink: /docs/adding-custom-executor
---

# Source Code of the Executor

The source code of the new executor should be at `executor/src/`.

1. Create a public executor unit struct.
2. Implement `TeaclaveExecutor` trait and `execute` function for the new struct.
3. Re-export your new executor in `executor/src/lib.rs` to make it callable.
4. Optionally, add unit test code to test your executor, and add a line calling your test in `executor/src/lib.rs`.

## Extern Function

Usually the new executor for other language or bytecode cannot be supported by this single rust source file,
and the embedded execution environment is ported from another project, which can be written in another
language. Therefore some extern functions should be imported to the rust source code and a **static** library
is needed in linking.

You may add this library in the linking command located at `cmake/scripts/sgx_link_sign.sh`, and such library
should be in `${TEACLAVE_OUT_DIR}`, which will be parsed to `build/intermediate` in build phase.

# Add the Interface

You also need to add some auxillary code for teaclave and tell it when and how to invoked the new executor.

## `types/src/worker.rs`

1. Add a new enum value in `ExecutorType`;
2. Add a match case in `ExecutorType::try_from` to get the `ExecutorType` from a string;
3. Add a match case in `ExecutorType::fmt` for printing;
4. Besides, add a enum in `Executor`;
5. Add match cases in `Executor::try_from` and `Executor::fmt` just like what you've done in step 3 and 4.

## `worker/src/worker.rs`

1. Import the executor in `use teaclave_executor::{...}`
2. Register the new executor in `Worker::Default`

# Invoke the New Executor

Just call the API and remember to set `executor_type` to your new executor type's name (the string used in
`ExecutorType::try_from` match case) when calling `register_function`, and set the `executor` to the executor's
name correspondingly.

