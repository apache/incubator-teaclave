# Implement your function

Example: mesatee_services/fns/sgx_trusted_lib/src/trusted_worker/private_join_and_compute.rs

## Write your function with rust

* Only use sgx-compatible crates.
* No I/O or network capabilities. 

## Function Types

* Single: Only one customer participates in this computation task
* Multiparty: More than two customers participate in this computation task

## Function entrypoint

**Convention**

```rust
fn function_name(
  helper: &mut WorkerHelper,   //crate::trait_defs::WorkerHelper
  input: WorkerInput,          //crate::trait_defs::WorkerInput
) -> Result<String>            //mesatee_core::Result
```

## Input

```rust
pub struct WorkerInput {
    pub function_name: String,     //function name
    pub input_files: Vec<String>,  //input file ids; files are saved in the TDFS
    pub payload: Option<String>,   //dynamic input, provided by the user who invokes the task
}
```

**Read a file with file id**

```rust
trait WorkerHelper {
    fn read_file(&mut self, file_id: &str) -> Result<Vec<u8>>;
}
```

**Note**: 

The function doesn't know who provides the payload/file. The file-id/payload can belong to any participatant in this task

## Output

There are four types of output

1. A String returned to the user who invokes the task. It's the return value of the function

   ```rust
   fn function_name(
     helper: &mut WorkerHelper,   //crate::trait_defs::WorkerHelper
     input: WorkerInput,          //crate::trait_defs::WorkerInput
   ) -> Result<String>            //mesatee_core::Result
   ```

2. Save the content to the TDFS for the task creator. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   ```rust
   trait WorkerHelper {
       fn save_file_for_task_creator(&mut self, data: &[u8]) -> Result<String>;
   }
   // return value is the file id
   ```

3. Save the content to the TDFS for the owner of one input file. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   ```rust
   trait WorkerHelper {
       fn save_file_for_file_owner(&mut self, data: &[u8], file_id: &str) -> Result<String>;
   }
   // return value is the file id
   ```

4. Save the content to the TDFS for all the participatants. 

   The file id will be saved to the task. The function return value doesn't need to contain the file id.

   Only the last one will be saved in the task.

   ```rust
   trait WorkerHelper {
   	fn save_file_for_all_participants(&mut self, data: &<u8>) -> Result<String>;
   }
   // return value is the file id
   ```

   

# Register your function

## Register in FunctionNodeService

```rust
In mesatee_services/fns/sgx_trusted_lib/src/trusted_worker/mod.rs
				dispatcher.insert(
            "function_name".to_string(),
            mod_name::function_name,
        );

```

## Register in Task Management Service

If the function is a multiparty function, it needs to be marked in the Task Management Service

```rust
In mesatee_services/tms/sgx_trusted_lib/src/tms_external.rs
	"psi" | "concat" | "swap_file" | "private_join_and_compute" | "your_function" => FunctionType::Multiparty
```

