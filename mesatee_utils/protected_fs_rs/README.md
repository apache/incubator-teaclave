# Rust bindings for ProtectedFS
`protected_fs_rs` is a rust binding for 
[protected_fs](https://github.com/intel/linux-sgx/tree/master/sdk/protected_fs) 
from the Intel SGX Linux SDK.

Beyond the original SGX-only implementations, `protected_fs_rs` now supports 
***running in both SGX and Non-SGX environment***. We ported the [original C 
implementations](https://github.com/intel/linux-sgx/tree/master/sdk/protected_fs
) in  `protected_fs_c` subdirectory and replaced the compile toolchains with 
CMake. Please refer to `build.rs` for more information.