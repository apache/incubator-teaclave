---
permalink: /docs/how-to-add-your-function
---

# Implement your own function

Currently, Teaclave supports two kinds of executors: native functions and Python
functions. Compared to Python functions, native functions have better
performance. Also, native functions can be statically linken into Teaclave
itself. So you can attest the implemented native function. 
In this document, we will show you how to add a native function to 
Teaclave stey by step with the example [`private_join_and_compute.rs`](https://github.com/apache/incubator-teaclave/blob/master/function/src/private_join_and_compute.rs). 

In the example, several banks have the name and the balance of its clients.
Because some clients may have several accounts in every bank. Those banks 
want to know the total amount of money if the user opens accounts in every bank.
If one client only has the account in one back. Then other banks cannot know
the client's any information during the computation. For example, if Bank A 
and Bank B do the computation. Then after the task, both Bank A and Bank B
know that client Bob has 3000 in Bank A and Bank B. However, Bank A doesn't
know Eva has an account in Bank B and Bank B doesn't know Alice has an account
in Bank B.

```
Bank A                  Bank B
Name : Balance          Name : Balance
Bob: 1000               Bob : 2000
Eva: 100                Alice : 100
```

Teaclave achieves the goal by asking those Banks encrypted the data and upload
the data into the trusted Teaclave executors. After both party attests the executors
and approve the task. The computation is done in the secure TEEs. Then each 
Bank retrived the information via an attested TLS channel.

## Write your function in Rust
The native function is written in Rust. Typically, we put the native
function under the folder `function/src`. 
You need to write down the name of your function and implement the
`run` method. The name is the identifier of the function. If you are going to
call the function, you need to pass the name of the function so
Teaclave can know which function is going to finish the computation.
`run` method is the like the main function for your task. It takes information
from the outside and returns the result.
We use `FunctionAruguments` to pass arguments into the function and
`FunctionRuntime` to read and write files. 
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
        ...
        Ok(summary)
}
```

To implement the `run` method, you may need to read the argument from `FunctionArguments`.
Before that, you can convert `FunctionArguments` into the structure you defined. For
example, here we define `PrivateJoinAndComputeArguments`. 
```rust
#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the multiple party computation
}
```
Typically, you can write `Tryfrom` trait to convert the `FunctionArguments` into the struct that
you define. Here we define the struct `PrivateJoinAndComputeArguments`.
In the example, we pass one parameter into the function, the number of banks that exchange the 
data.
We can read the parameter with the following code.
```rust
let args = PrivateJoinAndComputeArguments::try_from(arguments)?;
let num_user = args.num_user;
```
You can also read or write files by `FunctionRuntime`. To open a file, you need to
call the `open_input` method. To write a file, you need to call the `create_output`
method. Those files should be registerd by the client. After the computation, you can
write the result into file and return a summary of the result. It is mentioned the input
file should be encrypted with `cli` tool. After the computation, you can decrypt the 
output file with `cli` tool as well.Please refer the document of `cli` for further 
information.In the example, we write the result into the file and return a descriprion of the task.

## Rigister your function
After you implement your function, you need to register your function
in [`builtin.rs`](https://github.com/apache/incubator-teaclave/blob/master/executor/src/builtin.rs). 
As the source code of the builtin function is conditionally compiled using the attributes cfg.
You also need to update the `Cargo.toml` file.
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
            PrivateJoinAndCompute::NAME =>
            PrivateJoinAndCompute::new().run(arguments, runtime),
            ...
        }
    }
}
```
## Write the client
Currently, you can implement you client in Rust or Python. Examples can be found under
the `examples/python` and `tests/functional/enclave/src/end_to_end` directories.
For one function, you can have one client or several clients. If the task require
one client. Then the client should do the following step.

### Set up the task

### Register Input and Output files

### Approve the task

### Invoke the task

### Approve the task



