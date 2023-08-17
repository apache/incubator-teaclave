use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult, StatusCode},
    expr::GroupByMethod,
    types::{ExecutorRefId, FunctionArguments, OpaquePtr},
};

use super::*;

mod ffi {
    use super::*;

    extern "C" {
        #[linkage = "weak"]
        pub fn load_data(args: *const u8, args_len: usize) -> StatusCode;

        #[allow(unused)]
        #[linkage = "weak"]
        pub fn rustc_version(buf: *mut u8, len: *mut usize);

        #[linkage = "weak"]
        pub fn create_executor(
            args: *const u8,
            args_len: usize,
            p_executor: *mut OpaquePtr,
        ) -> StatusCode;

        #[linkage = "weak"]
        pub fn execute(executor: OpaquePtr, df_ptr: *mut u8, df_len: *mut usize) -> StatusCode;
    }
}

/// Loads the executors from the shared library and returns the id to these executors.
pub fn load_executor_lib(
    _path: &str,
    _args: FunctionArguments,
) -> PolicyCarryingResult<ExecutorRefId> {
    Ok(Default::default())
}

/// Tries to create a new executor from the loaded module with a given id indicating the executor set,
/// executor type, and the function arguments passed to the executor instance. This function returns an
/// opaque pointer to the executor allocated in the external library.
///
/// # Examples
///
/// ```
/// use policy_core::{args, types::ExecutorRefId};
/// use policy_ffi::create_executor;
///
/// trait Foo: Sized {
///     fn foo(&self) {
///         println!("Hello World!");
///     }
/// }
///
/// let executor =
///     create_executor(ExecutorRefId(0), ExecutorType::Filter, args!()).unwrap();
///
/// /* Do something with the opaque pointer `executor` */
///
/// ```
///
/// # FFI Safety
///
/// Rust provides no abi stablility, so we must compile the executor module and the core library by the
/// same Rust toolchain. Otherwise, the memory layout of the compiled trait object might appear different-
/// ly. This is a limitation.
///
/// Also, to guarantee that the trait object can be returned from the external library, we wrap it in a
/// nested [`Box`]: `Box<Box<dyn Executor>>` since fat pointers cannot be simply passed by FFI interfaces.
///
/// # Notes
///
/// The function is ignorant of the [`Box`]-ed object because [`Box`] is always [`Sized`]. As long as the
/// caller specifies `U` (which only accepts types whose sizes are determined at compilation time) as some
/// smart pointers that can carry a fat pointer, this function is safe (i.e., exhibits no undefined behavi-
/// or at runtime).
pub fn create_executor(
    _id: ExecutorRefId,
    args: FunctionArguments,
) -> PolicyCarryingResult<OpaquePtr> {
    let mut executor = std::ptr::null_mut();
    let args = serde_json::to_string(&args)
        .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))?;

    match unsafe { ffi::create_executor(args.as_ptr(), args.len(), &mut executor) } {
        StatusCode::Ok => Ok(executor),
        ret => Err(ret.into()),
    }
}

/// Tries to load the data from somewhere to the executor module.
pub fn load_data(_id: ExecutorRefId, args: FunctionArguments) -> PolicyCarryingResult<()> {
    let args = serde_json::to_string(&args)
        .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))?;
    match unsafe { ffi::load_data(args.as_ptr(), args.len()) } {
        StatusCode::Ok => Ok(()),
        err => Err(err.into()),
    }
}

/// Lets the executor do its task.
pub fn execute(_id: ExecutorRefId, ptr: OpaquePtr) -> PolicyCarryingResult<Vec<u8>> {
    let mut buf = vec![0; 1 << 16];
    let mut buf_len = 0usize;

    match unsafe { ffi::execute(ptr, buf.as_mut_ptr(), &mut buf_len) } {
        StatusCode::Ok => Ok(buf[..buf_len].to_vec()),
        err => Err(err.into()),
    }
}

pub fn get_udf(
    _id: ExecutorRefId,
    _ty: GroupByMethod,
) -> PolicyCarryingResult<UserDefinedFunction> {
    unimplemented!()
}
