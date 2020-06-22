---
permalink: /docs/builtin-functions
---

# How to Add Built-in Functions

There are several ways to execute user-defined functions in the Teaclave
platform. One simple way is to write Python scripts and register them as functions,
and the scripts will be executed by the *MesaPy executor*. Another way is to add native
functions as built-in functions, and they will be managed by the *Built-in executor*.
Compared to Python scripts, native built-in functions implemented in Rust are
memory-safe, have better performance, support more third-party libraries and
can be remotely attested as well. In this document, we will guide you through
how to add a built-in function to Teaclave step by step with a "private join and
compute" example.

In this example, consider several banks have names and balance of their clients.
These banks want to compute the total balance of common clients in their private
data set without leaking the raw sensitive data to other parties. This is
a perfect usage scenario of the Teaclave platform, and we will provide a
solution by implementing a built-in function in Teaclave.

## Implement Built-in Functions in Rust

All built-in functions are implemented in the `teaclave_function` crate and can
be selectively compiled using feature gates. Basically, one built-in function
needs two things: a name and a function implementation. Follow the convention of
other built-in function implementations, we define our "private join and
compute" function like this:

```rust
#[derive(Default)]
pub struct PrivateJoinAndCompute;

impl PrivateJoinAndCompute {
    pub const NAME: &'static str = "builtin-private-join-and-compute";
    pub fn new() -> Self {
        Default::default()
    }
    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> Result<String> {
        ...
        Ok(summary)
}
```

The `NAME` is the identifier of a function, which is used for creating tasks.
Usually, the name of a built-in function starts with the `built-in` prefix. In
addition, we need to define an entry point of the function, which is the `run`
function. The `run` function can take arguments (in the `FunctionAruguments`
type) and runtime (in the `FunctionRuntime` type) for interacting with external
resources (e.g., reading/writing input/output files). Also, the `run` function
can return a summary of the function execution.

Since the function arguments is in the JSON object format and can be easily
deserialized to a Rust struct with `serde_json`. Therefore, we define a struct
`PrivateJoinAndComputeArguments` which derive the `serde::Deserialize` trait for
the conversion. Then we implement `TryFrom` trait for the struct to convert the
`FunctionArguments` type to the actual `PrivateJoinAndComputeArguments` type.

```rust
#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the multiple party computation
}

impl TryFrom<FunctionArguments> for PrivateJoinAndComputeArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

```

When executing the function, a `runtime` object will be passed to the function.
We can read or write files with the `runtime` with the `open_input` and
`create_output` functions.


```rust
// Read data from a file
let mut input_io = runtime.open_input(&input_file_name)?;
input_io.read_to_end(&mut data)?;
...
// Write data into a file
let mut output = runtime.create_output(&output_file_name)?;
output.write_all(&output_bytes)?;
```

## Register Functions in the Executor

To use the function, we need to register it to the built-in executor. Please also
put a `cfg` attribute to make sure developers can conditionally build functions
into the executor.

``` rust
impl TeaclaveExecutor for BuiltinFunctionExecutor {
    fn execute(
        &self,
        name: String,
        arguments: FunctionArguments,
        _payload: String,
        runtime: FunctionRuntime,
    ) -> Result<String> {
        match name.as_str() {
            ...
            #[cfg(feature = "builtin_private_join_and_compute")]
            PrivateJoinAndCompute::NAME => PrivateJoinAndCompute::new().run(arguments, runtime),
            ...
        }
    }
}
```

## Invoke Functions with the Client SDK

Finally, we can invoke the function with the client SDK. In our example, we use
the Python client SDK. Basically, this process includes registering input/output
files, creating tasks, approving tasks, invoking tasks and getting execution
results. You can see more details in the `examples/python` directory.
