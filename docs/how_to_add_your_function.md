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

### Implement the `worker` trait for your worker.

```rust
trait Worker: Send + Sync {
    fn function_name(&self) -> &str;  // return function name
    fn function_type(&self) -> FunctionType; // return function type, Single/Multiparty
    fn set_id(&mut self, worker_id: u32); //id is set by FNS
    fn id(&self) -> u32;									//return id
    fn prepare_input(&mut self, dynamic_input: Option<String>, file_ids: Vec<String>)
        -> Result<()>;   //do some pre-processing for the input and save it to the structure
    fn execute(&mut self, context: WorkerContext) -> Result<String>; //this is the function to do the computation
}
```

### Example 

```rust
pub struct EchoWorker {
    worker_id: u32,
    func_name: String,
    func_type: FunctionType,
    input: Option<EchoWorkerInput>,
}
impl EchoWorker {
    pub fn new() -> Self {
        EchoWorker {
            worker_id: 0,
            func_name: "echo".to_string(),
            func_type: FunctionType::Single,
            input: None,
        }
    }
}
struct EchoWorkerInput {
    msg: String,
}
impl Worker for EchoWorker {
    fn function_name(&self) -> &str {
        self.func_name.as_str()
    }
    fn function_type(&self) -> FunctionType {
        self.func_type
    }
    fn set_id(&mut self, worker_id: u32) {
        self.worker_id = worker_id;
    }
    fn id(&self) -> u32 {
        self.worker_id
    }
    fn prepare_input(
        &mut self,
        dynamic_input: Option<String>,
        _file_ids: Vec<String>,
    ) -> Result<()> {
        let msg = dynamic_input.ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        self.input = Some(EchoWorkerInput { msg });
        Ok(())
    }
    fn execute(&mut self, _context: WorkerContext) -> Result<String> {
        let input = self
            .input
            .take()
            .ok_or_else(|| Error::from(ErrorKind::InvalidInputError))?;
        Ok(input.msg)
    }
}
```



## Input

MesaTEE will provide the worker with two types of input: 

* Dynamic Input, provided by the client who invokes the task
* File ids, provided by clients during task creation and approval

You can do some pre-processing and prepare your own input structure.

```rust
impl Worker for MyWorker {  
    fn prepare_input(
        dynamic_input: Option<String>, // dynamic input
        file_ids: Vec<String>,         //input file ids; files are saved in the TDFS
    ) -> Result<()> {
        unimplemented!();
        // Save the input to the structure
    }
}
```

**Read a file with file id**

Since some input are file ids, during task execution, the worker can read the data with the file id. 

```rust
impl WorkerContext {
    fn read_file(&self, file_id: &str) -> Result<Vec<u8>>;
}
pub fn read_file(context_id: &str, context_token: &str, file_id: &str) -> Result<Vec<u8>>;
// The above apis are equivalent
```

**Note**: 

The function doesn't know who provides the payload/file. The file-id/dynamic-input can belong to any participatant in this task

## Output

There are four types of output

1. A String returned to the client who invokes the task. It's the return value of the ``execute`` function

   ```rust
   impl Worker for MyWorker { 
   	fn execute(&mut self, context: WorkerContext) -> Result<String> 
   }
   ```
   
2. Save the content to the TDFS for the task creator. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   ```rust
   impl WorkerContext {
       pub fn save_file_for_task_creator(&self, data: &[u8]) -> Result<String>;
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
pub fn register_trusted_worker_statically() {
  for _i in 0..number_of_worker_instances {
      let worker = Box::new(MyWorker::new());
      let _ = WorkerInfoQueue::register(worker);
  }
} 
```

## Register in Task Management Service

If the function is a multiparty function, it needs to be marked in the Task Management Service

```rust
In mesatee_services/tms/sgx_trusted_lib/src/tms_external.rs
	"psi" | "concat" | "swap_file" | "private_join_and_compute" | "my_function" => FunctionType::Multiparty
```

