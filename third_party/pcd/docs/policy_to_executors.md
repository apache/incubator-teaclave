# Building Executors from Policy

We want to de-couple the implementation of the data analysis framework and the physical executor set that eventually enforces the policy. This has two major benefits. First, we reduce the codebase that should be verified, making verification simpler and more concise. Second, we reduce the code that will be included in the TCB. Therefore, we move the executors as a seperate pluggable module (if implemented as a dynamic loadable library) or a seperate served TEE container. By introducing a global manager that keeps track of the active executors, we dispatch the constructed physical plans to thedynamically loadable executors which do the job for us, and the users are completely ignorant of this behavior.

The executors that enforce the policy-compliant data access behavior are made as a separate module which the users can manipulate via an opaque pointers created at the first invocation of `create_executor` function. In future design, we will also support the remote process call to invoke necessary executors that may reside inside a TEE that provides stronger security guarantees.

# Layout of an executor library

By default, the data analysis library will try to load the module and find two symbols to create the executors and load the necessary data structures. The first function is called `on_load`, which is called upon the library is loaded. There is also an `on_unload`. The prototypes are

```rust
#[no_mangle]
extern "C" fn on_load(args: *const u8, args_len: usize) -> StatusCode;

#[no_mangle]
extern "C" fn on_unload(args: *const u8, args_len: usize) -> StatusCode;
```

This function can be regarded as a prelogue function. Any preparation tasks are fulfilled by this function.

Yet there is another function called `create_executor` whose prototype is

```rust
#[no_mangle]
extern "C" fn create_executor(
    executor_type: u64,
    args: *const u8,
    args_len: usize,
    p_executor: *mut OpaquePtr,
) -> StatusCode;
```

This function creates new executors on demand of the data analysis framework when the query is evaluated at the physical level.

## Examples

The `examples` folder contains the sample usage of compiling the executors into a standalong shared library, which can be later loaded by the program. The `simple_executor` implements the minimal set of executors (yet no policy enforcement is implemented) that can be dynamically loaded to the data analysis framework, and `executor_user` is the application that performs the data analysis job.

## Building Executors for SGX Enclaves

Since there is no official Rust standard library (`std`) for enclave environment, one should only use the Teaclave's Rust SGX SDK to build the default std library and link it against the executors. There are some known issues:

* Building executors as a standalone static library using custom sysroot (the `std` library built from SGX SDK) and linking it against the enclave will cause duplicate symbols like `__rust_eh_personality`. For this issue, there is no good solution because we cannot "hide" some `std` symbols.
* Building executors as a shared object and linking it against the enclave (not tested, may work).
