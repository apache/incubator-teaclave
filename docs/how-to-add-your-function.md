---
permalink: /docs/how-to-add-your-function
---

# Implement your own function

Example: function/src/private_join_and_compute.rs

## Define function with rust
You need to write down the name of your function and implement the
`run` method. `run` method is the main boady of your function.
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
```rust
#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the mutiple party computation
}
```
Here we only pass one parameter into the function, you can read the parameter.
```rust
let args = PrivateJoinAndComputeArguments::try_from(arguments)?;
let num_user = args.num_user;
```
You can also read or write files by `FunctionRuntime`. 

## Rigister your function
After you implement your function, you need to register your function
in [`builtin.rs`](https://github.com/apache/incubator-teaclave/blob/master/executor/src/builtin.rs). As the source code of the builtin is conditionally compiled using the attributes cfg. You also need to update the `Cargo.toml` file.

## Write the client
Currently, you can implement you client in Rust or Python. Examples can be found under
the `examples/python` and `tests/functional/enclave/src/end_to_end` directories.

