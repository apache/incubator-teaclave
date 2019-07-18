# Implement your worker

Example: mesatee_services/fns/sgx_trusted_lib/src/trusted_worker/private_join_and_compute.rs

## Write your worker with rust

* Only use sgx-compatible crates.
* No I/O or network capabilities. 
* Handle the errors properly

## Function Types

* Single: Only one customer participates in this computation task
* Multiparty: More than two customers participate in this computation task

## Worker Definition

```rust
pub struct YourWorker {}
pub struct YourWorkerWorkerInput {
    field_one: field_type,
    field_two: field_type
}
impl Worker for YourWorker {
    type InputType = YourWorkerWorkerInput;
    fn new(worker_info: &WorkerInfo) -> Self {
        unimplemented!();
    }
    fn get_function_info() -> FunctionInfo {
        FunctionInfo {
            funtion_name: "function_name".to_string(),
            function_type: FunctionType::Single, // or FunctionType::Multiparty
        }
    }
    fn prepare_input(
        dynamic_input: Option<String>,
        file_ids: Vec<String>,
    ) -> Result<Self::InputType> {
        unimplemented!();
    }

  	// this is the function to do the computation
    fn execute(&mut self, input: Self::InputType, context: WorkerContext) -> Result<String> {
        unimplemented!();    
    }
}
```

## Input

MesaTEE will provide the worker with two types of input: 

* Dynamic Input, provided by the client who invokes the task
* File ids, provided by clients during task creation and approval

You can do some pre-processing and prepare your own input structure.

```rust
impl Worker for YourWorker {  
    type InputType = YourWorkerWorkerInput;
    fn prepare_input(
        dynamic_input: Option<String>, // dynamic input
        file_ids: Vec<String>,         //input file ids; files are saved in the TDFS
    ) -> Result<Self::InputType> {
        unimplemented!();
    }
}
```

**Read a file with file id**

Since some input are file ids, during task execution, the worker can read the data with the file id. 

```rust
impl WorkerContext {
    fn read_file(&self, file_id: &str) -> Result<Vec<u8>>;
}
pub fn read_file(context_id: &str, context_token: &str, file_id: &str) -> Result<Vec<u8>> {
}
// The above apis are equivalent
```

**Note**: 

The function doesn't know who provides the payload/file. The file-id/dynamic-input can belong to any participatant in this task

## Output

There are four types of output

1. A String returned to the user who invokes the task. It's the return value of the ``execute`` function

   ```rust
   impl Worker for YourWorker { 
   	fn execute(&mut self, input: Self::InputType, context: WorkerContext) -> Result<String> 
   }
   ```
   
2. Save the content to the TDFS for the task creator. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   ```rust
   impl WorkerContext {
       pub fn save_file_for_task_creator(&self, data: &[u8]) -> Result<String> {
   }
   pub fn save_file_for_task_creator(
       context_id: &str,
       context_token: &str,
       data: &[u8],
   ) -> Result<String>
   // return value is the file id
   ```

3. Save the content to the TDFS for the owner of one input file. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   ```rust
   impl WorkerContext {
       pub fn save_file_for_file_owner(&self, data: &[u8], file_id: &str) -> Result<String> 
   }
   pub fn save_file_for_file_owner(
       context_id: &str,
       context_token: &str,
       data: &[u8],
       file_id: &str,
   ) -> Result<String>
   // return value is the file id
   ```

4. Save the content to the TDFS for all the participatants. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   Only the last one will be saved in the task.

   ```rust
   impl WorkerContext {
       pub fn save_file_for_all_participants(&self, data: &[u8]) -> Result<String>
   }
   pub fn save_file_for_all_participants(
       context_id: &str,
    context_token: &str,
       data: &[u8],
   ) -> Result<String>
   // return value is the file id
   ```
   
   

# Register your worker

## Register in FunctionNodeService

####Statically register your worker

```rust
In mesatee_services/fns/sgx_trusted_lib/src/global.rs
   You need to provide the *Worker Structure* and *max concurrent number* 
pub fn register_trusted_worker_statically() {
    let function_info = YourWorker::get_function_info();
    let _ = register_trusted_worker(&function_info, MAX_CONCURRENT_NUMBER);
}
```

#### Add dispatcher code 

```rust
In mesatee_services/fns/sgx_trusted_lib/src/fns.rs
   You need to provide the *function name* and the *Worker Structure*
fn invoke_worker(
    worker_info: &WorkerInfo,
    worker_context: WorkerContext,
    function_name: &str,
    dynamic_input: Option<String>,
    file_list: Vec<String>,
) -> Result<String> {
    match function_name {
      "your function name" => {
            let mut worker = YourWorker::new(&worker_info);
            let input = YourWorker::prepare_input(dynamic_input, file_list)?;
            worker.execute(input, worker_context)
      }
  }
}
```

#### 

## Register in Task Management Service

If the function is a multiparty function, it needs to be marked in the Task Management Service

```rust
In mesatee_services/tms/sgx_trusted_lib/src/tms_external.rs
	"psi" | "concat" | "swap_file" | "private_join_and_compute" | "your_function" => FunctionType::Multiparty
```

