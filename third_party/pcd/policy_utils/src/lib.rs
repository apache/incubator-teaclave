use std::sync::Arc;

use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    types::FunctionArguments,
};

/// Moves `T` out of the pointer to the [`Box`]-ed `T`.
pub fn move_box_ptr<T: ?Sized>(box_ptr: *mut Box<T>) -> Box<T> {
    unsafe { *Box::from_raw(box_ptr) }
}

/// Moves `T` out of the pointer to the [`Arc`]-ed `T`.
pub fn move_arc_ptr<T: ?Sized>(box_ptr: *mut Arc<T>) -> Arc<T> {
    unsafe { *Box::from_raw(box_ptr) }
}

pub fn args_from_raw(args: *const u8, args_len: usize) -> PolicyCarryingResult<FunctionArguments> {
    // Deserialize the arguments.
    let args = unsafe {
        let args = std::slice::from_raw_parts(args, args_len);
        std::str::from_utf8_unchecked(args)
    };
    serde_json::from_str::<FunctionArguments>(args)
        .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
}
