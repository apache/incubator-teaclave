---
permalink: /docs/how-to-add-your-function
---

# Implement your own function

Example: function/src/private_join_and_compute.rs

Currently, Teaclave supports two kinds of executors: native functions and Python
functions. In order to support better performance, you can implement you own
functions in Rust. 

## Define function with rust
You need to write down the name of your function and implement the
`run` method. `run` method is the main body of your function. It 
takes input from `FunctionArguments` and `FunctionRuntime`.
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
    ) -> anyhow::Result<String> {
}
```

To implement the `run` method, you may need to read the argument from `FunctionArguments`.
Before that, you need to define the structure of the `FunctionArguments` and convert the
`Json` string into the structure.
```rust
#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the multiple party computation
}
```
Here we only pass one parameter into the function, you can read the parameter with the 
following code.
```rust
let args = PrivateJoinAndComputeArguments::try_from(arguments)?;
let num_user = args.num_user;
```
You can also read or write files by `FunctionRuntime`. To open a file, you need to
call the `open_input` method. To write a file, you need to call the `create_output`
method.

## Rigister your function
After you implement your function, you need to register your function
in [`builtin.rs`](https://github.com/apache/incubator-teaclave/blob/master/executor/src/builtin.rs). 
As the source code of the builtin function is conditionally compiled using the attributes cfg.
 You also need to update the `Cargo.toml` file.

## Write the client
Currently, you can implement you client in Rust or Python. Examples can be found under
the `examples/python` and `tests/functional/enclave/src/end_to_end` directories.

