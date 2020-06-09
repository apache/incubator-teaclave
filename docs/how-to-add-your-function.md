---
permalink: /docs/how-to-add-your-function
---

# Implement your own function

Teaclave supports two kinds of executors: native functions and Python
functions. Compared to Python functions, native functions have better
performances. As native functions can be statically linken into Teaclave
itself, you can attest the implemented native function as well. 
In this document, we will show you how to add a native function to 
Teaclave stey by step with the example from [`private_join_and_compute.rs`](https://github.com/apache/incubator-teaclave/blob/master/function/src/private_join_and_compute.rs). 

In the example, several banks have the name and the balance of their clients.
Because some clients may have several accounts in every bank. Those banks 
want to know the total amount of money if the client opens accounts in every bank.
If one client only has one account in one back. Then other banks cannot know
the client's any information during the computation. For example, if Bank A 
and Bank B do the computation. Then after the task, both Bank A and Bank B
know that client Bob has 3000 dollars in Bank A and Bank B. However, Bank A doesn't
know Eva has an account in Bank B and Bank B doesn't know Alice has an account
in Bank B.

```
Bank A                  Bank B
Name : Balance          Name  : Balance
Bob  : 1000             Bob   : 2000
Eva  : 100              Alice : 100
```

Teaclave achieves the goal by asking those Banks to upload the encrypted data into the trusted Teaclave executors. After both party attests the executors
and approve the task. The computation is done in the secure TEEs. 
So the confidentiality and integrity of the data is protected. 
After the computation. Each 
Bank retrives the encrypted result via an attested TLS channel.

## Write your function in Rust
The native function is written in Rust. Typically, we put the native
function under the folder `function/src`. 
You need to write down the name of your function and implement the
`run` method. The name is the identifier of the function. If you are going to
call the function, you need to pass the name of the function, so
Teaclave can know which function is going to finish the computation.
`run` method is like the main function for your task. It takes information
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
Before that, you can convert `FunctionArguments` into the structure you defined to ease
the implementation. For
example, here we define the struct `PrivateJoinAndComputeArguments`. 
```rust
#[derive(serde::Deserialize)]
struct PrivateJoinAndComputeArguments {
    num_user: usize, // Number of users in the multiple party computation
}
```
Typically, you can write `Tryfrom` trait to convert the `FunctionArguments` into the struct that
you define. 
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
write the result into file and return a summary of the result. It is mentioned that the input
file should be encrypted with `cli` tool. After the computation, you can decrypt the 
output file with `cli` tool as well. Please refer the document of `cli` for further 
information. In the example, we write the result into the file and return a descriprion of the task. However, Teaclave can do the encryption and decryption autimatically for you,
so you can think those files as the plaintext.
```rust
// Read data from a file
let mut input_io = runtime.open_input(&input_file_name)?;
input_io.read_to_end(&mut data)?;
...
// Write data into a file
let mut output = runtime.create_output(&output_file_name)?;
output.write_all(&output_bytes)?;
```

## Rigister your function
After you finish implementing your function, you need to register your function
in [`builtin.rs`](https://github.com/apache/incubator-teaclave/blob/master/executor/src/builtin.rs). 
As the source code of the builtin function is conditionally compiled using the attributes `cfg`.
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
Currently, you can implement your client in Rust or Python. Examples can be found under
the `examples/python` and `tests/functional/enclave/src/end_to_end` directories.
For one task, Teaclave have one party or several parties. In the example, because we
may have several banks that want to exachange the data. The task requires
the involvement of several parties.
Still, we use the client of [`private_join_and_compute.rs`](https://github.com/apache/incubator-teaclave/blob/master/function/src/private_join_and_compute.rs) as an example.
It is located at the `examples/python` folder.

In the example, we have four users in total. User 3 creates the task and user 0, 1, 2
upload their data. After the computation, user 0, 1, 2 can get the result. It is mentioned
that we have a seperate user (user 3) in the example to illustrate the problem
clearly. In reality, one user can create the task and upload the data at the same time.

#### Set up the task
The first step of task is to register the function. Here User 3 creates the task and defines
the input and output informaiton. `name` is the function name we write in the native
function. Becasue it is a native function, so `executor_type` is `builts`. We also define
the argument, input files, and output files of the task. 
```Python
function_id = client.register_function(
    name="builtin-private-join-and-compute",
    description="Native Private Join And Compute",
    executor_type="builtin",
    arguments=["num_user"],
    inputs=[
        FunctionInput("input_data0", "Bank A data file."),
        FunctionInput("input_data1", "Bank B data file."),
        FunctionInput("input_data2", "Bank C data file.")
    ],
    outputs=[
        FunctionOutput("output_data0", "Output data."),
        FunctionOutput("output_data1", "Output data."),
        FunctionOutput("output_data2", "Output date.")
        ])
```
After that, we can create the task. For the example, we have three users who share the data in total. We also define the ownership of the file. That is, only the user who own the file can
read or wirte the file. We then get the `task_id`, the identifier of the task.
```Python
task_id = client.create_task(function_id=function_id,
                             function_arguments=({
                                "num_user": 3,
                             }),
                             executor="builtin",
                             inputs_ownership=[
                                OwnerList("input_data0",
                                          [user0_data.user_id]),
                                OwnerList("input_data1",
                                          [user1_data.user_id]),
                                OwnerList("input_data2",
                                          [user2_data.user_id])
                                ],
                            outputs_ownership=[
                                OwnerList("output_data0",
                                          [user0_data.user_id]),
                                OwnerList("output_data1",
                                          [user1_data.user_id]),
                                OwnerList("output_data2",
                                          [user2_data.user_id])
                            ])
```

#### Register Input and Output files
After the task is created. Users need to register the input and output files. In the example, User 0, 1, 2 reigister the input and output file. One user register one input
file and one output file. It is mentioned only the user who owns the file can register the 
file.
```Python
data_id = client.register_input_file(url, schema, key, iv,
                                     cmac)
output_id = client.register_output_file(url, schema, key, iv)
...
client.assign_data_to_task(task_id,
                           [DataList(input_label, training_id)],
                           [DataList(output_label, output_id)])
```
#### Approve the task
Before invoking the task, Teaclave requires every party involving in the task approves the task.
```Python
user0.approve_task(task_id)
```
#### Invoke the task
The next step is to invoke the task. Because every party involving in the task has approved 
the task. Teaclave only require one party to invoke the task. If one party doesn't approve
the task, the step will fail.
In the example, user 3 invokes the task.

#### Get the result
After the computation, one user can get results from two sources, a summary of the task you returned by your implemented function and the output files you register in the above task. 
The summary of the task is shared by all users. It is usually a description of the 
computation task. In the example, all users get the output file with the same content. 
But different users can get different output files as well. 

The example of the client in the document can be found in [`private_join_and_compute.py`](https://github.com/apache/incubator-teaclave/blob/dev/examples/python/builtin_private_join_and_compute.py).

