use policy_carrying_data::{
    api::{ApiRequest, PolicyApiSet},
    DataFrame,
};
use policy_core::error::{PolicyCarryingError, PolicyCarryingResult};
use std::{pin::Pin, sync::Arc};

#[no_mangle]
extern "C" fn load_module(name: *const u8, len: usize, ptr: *mut u64) -> i32 {
    let name = unsafe {
        let str = std::slice::from_raw_parts(name, len);
        std::str::from_utf8_unchecked(str)
    };

    if name != PLUGIN_NAME {
        eprintln!("error: loading a wrong module");
        // Error!
        1
    } else {
        // Double pointer to ensure that we do not lose information in a fat pointer.
        let wrapped = Box::pin(Arc::new(DiagnosisDataPolicy::new()) as Arc<dyn PolicyApiSet>);

        unsafe {
            // Consume the box and leave the ownership to the caller.
            *ptr = Box::into_raw(Pin::into_inner_unchecked(wrapped)) as u64;
        }

        0
    }
}

static PLUGIN_NAME: &str = "DiagnosisDataPolicy";

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct DiagnosisDataPolicy;

impl DiagnosisDataPolicy {
    fn new() -> Self {
        Self::default()
    }
}

impl PolicyApiSet for DiagnosisDataPolicy {
    fn name(&self) -> &'static str {
        PLUGIN_NAME
    }

    fn load(&self) {}

    fn unload(&self) {}

    fn entry(&self, req: ApiRequest) -> PolicyCarryingResult<DataFrame> {}
}
