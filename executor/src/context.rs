// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::cell::RefCell;
use std::slice;
use std::thread_local;

use sgx_types::{c_char, c_int, c_uchar, c_uint, size_t};

use std::collections::HashMap;
use std::format;

use teaclave_types::TeaclaveRuntime;

use std::ffi::c_void;

const FFI_OK: c_uint = 0;
const FFI_FILE_ERROR: c_uint = 1;
const FFI_FILE_ERROR_WASM: c_int = -1;

pub struct Context {
    runtime: Box<dyn TeaclaveRuntime + Send + Sync>,
    seq: Sequence,
    read_handles: HandleRegistry<Box<dyn std::io::Read>>,
    write_handles: HandleRegistry<Box<dyn std::io::Write>>,
}

impl Context {
    pub fn new(runtime: Box<dyn TeaclaveRuntime + Send + Sync>) -> Context {
        Context {
            runtime,
            seq: Sequence::new(1, 1024),
            read_handles: HandleRegistry::default(),
            write_handles: HandleRegistry::default(),
        }
    }

    fn open_input(&mut self, fid: &str) -> anyhow::Result<FileHandle> {
        let file = self.runtime.open_input(fid)?;
        let handle = self.seq.next()?.into_read_handle();
        self.read_handles.add(handle, file)?;
        Ok(handle)
    }

    fn create_output(&mut self, fid: &str) -> anyhow::Result<FileHandle> {
        let file = self.runtime.create_output(fid)?;
        let handle = self.seq.next()?.into_write_handle();
        self.write_handles.add(handle, file)?;
        Ok(handle)
    }

    fn read_handle(&mut self, handle: FileHandle, buf: &mut [u8]) -> anyhow::Result<usize> {
        let file = self.read_handles.get_mut(handle)?;
        let size = file.read(buf)?;
        Ok(size)
    }

    fn write_handle(&mut self, handle: FileHandle, buf: &[u8]) -> anyhow::Result<usize> {
        let file = self.write_handles.get_mut(handle)?;
        let size = file.write(buf)?;
        Ok(size)
    }

    fn close_handle(&mut self, handle: FileHandle) -> anyhow::Result<()> {
        if handle.is_read_handle() {
            self.read_handles.remove(handle)?;
        } else {
            self.write_handles.remove(handle)?;
        }
        Ok(())
    }
}

trait HandleEncoding {
    fn into_write_handle(self) -> FileHandle;
    fn into_read_handle(self) -> FileHandle;
    fn is_write_handle(&self) -> bool;
    fn is_read_handle(&self) -> bool;
}

impl HandleEncoding for FileHandle {
    fn into_write_handle(self) -> FileHandle {
        assert!(self < HANDLE_UPPDER_BOUND);
        0x4000_0000 | self
    }

    fn is_write_handle(&self) -> bool {
        0x4000_0000 & self > 0
    }

    fn into_read_handle(self) -> FileHandle {
        assert!(self < HANDLE_UPPDER_BOUND);
        self
    }

    fn is_read_handle(&self) -> bool {
        !self.is_write_handle()
    }
}

struct Sequence {
    range: std::ops::Range<FileHandle>,
}

impl Sequence {
    fn new(start: FileHandle, end: FileHandle) -> Self {
        Sequence {
            range: (start..end),
        }
    }

    fn next(&mut self) -> anyhow::Result<FileHandle> {
        self.range
            .next()
            .ok_or_else(|| anyhow::anyhow!("Reached max sequence"))
    }
}

type FileHandle = i32;
const HANDLE_UPPDER_BOUND: FileHandle = 0x1000_0000;

struct HandleRegistry<T> {
    entries: HashMap<FileHandle, T>,
}

impl<T> HandleRegistry<T> {
    fn add(&mut self, handle: FileHandle, obj: T) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.entries.get(&handle).is_none(),
            "Reuse a existed handle: {}",
            handle
        );
        self.entries.insert(handle, obj);
        Ok(())
    }

    fn get_mut(&mut self, handle: FileHandle) -> anyhow::Result<&mut T> {
        self.entries
            .get_mut(&handle)
            .ok_or_else(|| anyhow::anyhow!("Get an invalid handle: {}", handle))
    }

    fn remove(&mut self, handle: FileHandle) -> anyhow::Result<()> {
        self.entries
            .remove(&handle)
            .ok_or_else(|| anyhow::anyhow!("Remove an invalid handle: {}", handle))?;
        Ok(())
    }
}

impl<T> std::default::Default for HandleRegistry<T> {
    fn default() -> Self {
        HandleRegistry {
            entries: HashMap::<FileHandle, T>::new(),
        }
    }
}

thread_local! {
    pub static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub fn reset_thread_context() -> anyhow::Result<()> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        *ctx = None;
        Ok(())
    })
}

pub fn set_thread_context(context: Context) -> anyhow::Result<()> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_none(), "Context already initialized");
        *ctx = Some(context);
        Ok(())
    })
}

pub fn rtc_open_input(fid: &str) -> anyhow::Result<FileHandle> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        ctx.as_mut().unwrap().open_input(fid)
    })
}

pub fn rtc_create_output(fid: &str) -> anyhow::Result<FileHandle> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        ctx.as_mut().unwrap().create_output(fid)
    })
}

pub fn rtc_read_handle(f: FileHandle, buf: &mut [u8]) -> anyhow::Result<usize> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        ctx.as_mut().unwrap().read_handle(f, buf)
    })
}

pub fn rtc_write_handle(f: FileHandle, buf: &[u8]) -> anyhow::Result<usize> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        ctx.as_mut().unwrap().write_handle(f, buf)
    })
}

pub fn rtc_close_handle(f: FileHandle) -> anyhow::Result<()> {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        anyhow::ensure!(ctx.is_some(), "Context not initialized");
        ctx.as_mut().unwrap().close_handle(f)
    })
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;
    use teaclave_crypto::TeaclaveFile128Key;
    use teaclave_runtime::RawIoRuntime;
    use teaclave_test_utils::*;
    use teaclave_types::hashmap;
    use teaclave_types::FileAuthTag;
    use teaclave_types::StagedFileInfo;
    use teaclave_types::StagedFiles;

    pub fn run_tests() -> bool {
        run_tests!(test_file_handle_encoding, test_rtc_api,)
    }

    fn test_file_handle_encoding() {
        assert_eq!(5, 5.into_read_handle());
        assert_eq!(0x4000_0006, 6.into_write_handle());
        assert_eq!(true, 0x4000_0000.is_write_handle());
        assert_eq!(false, 0x4000_0000.is_read_handle());
        assert_eq!(true, 0x9.is_read_handle());
        assert_eq!(false, 0xff.is_write_handle());
    }

    fn test_rtc_api() {
        let input = PathBuf::from_str("fixtures/functions/mesapy/input.txt").unwrap();
        let output = PathBuf::from_str("fixtures/functions/mesapy/output.txt.out").unwrap();

        let input_info =
            StagedFileInfo::new(input, TeaclaveFile128Key::random(), FileAuthTag::mock());
        let output_info =
            StagedFileInfo::new(output, TeaclaveFile128Key::random(), FileAuthTag::mock());

        let in_fid = "in_f1";
        let out_fid = "out_f1";
        let input_files = StagedFiles::new(hashmap!(in_fid => input_info));
        let output_files = StagedFiles::new(hashmap!(out_fid => output_info));

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        set_thread_context(Context::new(runtime)).unwrap();

        let expected_input = b"Hello\nWorld";
        let f = rtc_open_input(&in_fid).unwrap();
        let mut buf = [0u8; 128];
        let size = rtc_read_handle(f, &mut buf).unwrap();
        assert_eq!(&expected_input[..], &buf[..size]);

        assert!(rtc_close_handle(f).is_ok());
        assert!(rtc_close_handle(f).is_err());

        let f = rtc_create_output(&out_fid).unwrap();
        let size = rtc_write_handle(f, &expected_input[..]).unwrap();
        assert_eq!(size, expected_input.len());

        assert!(rtc_close_handle(f).is_ok());
        assert!(rtc_close_handle(f).is_err());
        reset_thread_context().unwrap();
    }
}

use std::ffi::CStr;

/*
 * uint c_open_input(char* file_id, int* out_fd);
 */
#[allow(unused)]
#[no_mangle]
extern "C" fn c_open_input(fid: *mut c_char, out_handle: *mut c_int) -> c_uint {
    debug!("c_open_input");
    let fid = unsafe { CStr::from_ptr(fid).to_string_lossy().into_owned() };
    match rtc_open_input(&fid) {
        Ok(handle) => {
            unsafe {
                *out_handle = handle;
            }
            FFI_OK
        }
        Err(e) => {
            error!("c_open_file: {:?}", e);
            FFI_FILE_ERROR
        }
    }
}

/*
 * int teaclave_open_input(char* file_id);
 */
#[allow(unused)]
#[no_mangle]
pub extern "C" fn wasm_open_input(_exec_env: *const c_void, fid: *mut c_char) -> c_int {
    debug!("wasm_open_input");
    let fid = unsafe { CStr::from_ptr(fid).to_string_lossy().into_owned() };
    match rtc_open_input(&fid) {
        Ok(handle) => handle as i32,
        Err(e) => {
            error!("wasm_open_input: {:?}", e);
            FFI_FILE_ERROR_WASM
        }
    }
}

/*
 * uint c_create_output(char* file_id, int* out_fd);
 */
#[allow(unused)]
#[no_mangle]
extern "C" fn c_create_output(fid: *mut c_char, out_handle: *mut c_int) -> c_uint {
    debug!("c_create_input");
    let fid = unsafe { CStr::from_ptr(fid).to_string_lossy().into_owned() };
    match rtc_create_output(&fid) {
        Ok(handle) => {
            unsafe {
                *out_handle = handle;
            }
            FFI_OK
        }
        Err(e) => {
            error!("c_open_file: {:?}", e);
            FFI_FILE_ERROR
        }
    }
}

/*
 * int teaclave_create_output(char* file_id);
 *
 */

#[allow(unused)]
#[no_mangle]
pub extern "C" fn wasm_create_output(_exec_env: *const c_void, fid: *mut c_char) -> c_int {
    debug!("wasm_create_output");
    let fid = unsafe { CStr::from_ptr(fid).to_string_lossy().into_owned() };
    match rtc_create_output(&fid) {
        Ok(handle) => handle as i32,
        Err(e) => {
            error!("wasm_create_output: {:?}", e);
            FFI_FILE_ERROR_WASM
        }
    }
}

/*
 * uint c_read_file(int fd, void* out_buf, size_t buf_size, size_t* out_size_read);
 */
#[allow(unused)]
#[no_mangle]
extern "C" fn c_read_file(
    handle: c_int,
    out_buf: *mut c_uchar,
    buf_size: size_t,
    out_buf_size_p: *mut size_t,
) -> c_uint {
    debug!("c_read_file");
    let out: &mut [u8] = unsafe { slice::from_raw_parts_mut(out_buf, buf_size) };

    match rtc_read_handle(handle, out) {
        Ok(size) => {
            unsafe {
                *out_buf_size_p = size;
            }
            FFI_OK
        }
        Err(e) => {
            error!("c_read_file: {:?}", e);
            FFI_FILE_ERROR
        }
    }
}

/*
 * int teaclave_read_file(int fd, void* out_buf, int buf_size);
 */
#[allow(unused)]
#[no_mangle]
pub extern "C" fn wasm_read_file(
    _exec_env: *const c_void,
    handle: c_int,
    out_buf: *mut c_uchar,
    buf_size: c_int,
) -> c_int {
    debug!("wasm_read_file");
    let out: &mut [u8] = unsafe { slice::from_raw_parts_mut(out_buf, buf_size as usize) };

    match rtc_read_handle(handle, out) {
        Ok(size) => size as i32,
        Err(e) => {
            error!("wasm_read_file: {:?}", e);
            FFI_FILE_ERROR_WASM
        }
    }
}

/*
 * uint c_write_file(int fd, void* buf, size_t buf_size, size_t* out_size_written);
 */
#[allow(unused)]
#[no_mangle]
extern "C" fn c_write_file(
    handle: c_int,
    in_buf: *mut c_uchar,
    buf_size: size_t,
    buf_size_p: *mut size_t,
) -> c_uint {
    debug!("c_write_file");
    let in_buf: &[u8] = unsafe { slice::from_raw_parts_mut(in_buf, buf_size) };

    match rtc_write_handle(handle, in_buf) {
        Ok(size) => {
            unsafe {
                *buf_size_p = size;
            }
            FFI_OK
        }
        Err(e) => {
            error!("c_write_file: {:?}", e);
            FFI_FILE_ERROR
        }
    }
}

/*
 * int teaclave_write_file(int fd, void* buf, size_t buf_size);
 */
#[allow(unused)]
#[no_mangle]
pub extern "C" fn wasm_write_file(
    _exec_env: *const c_void,
    handle: c_int,
    in_buf: *mut c_uchar,
    buf_size: c_int,
) -> c_int {
    debug!("wasm_write_file");
    let in_buf: &[u8] = unsafe { slice::from_raw_parts_mut(in_buf, buf_size as usize) };

    match rtc_write_handle(handle, in_buf) {
        Ok(size) => size as i32,
        Err(e) => {
            error!("wasm_write_file: {:?}", e);
            FFI_FILE_ERROR_WASM
        }
    }
}

/*
 * uint c_close_file(int fd);
 */
#[allow(unused)]
#[no_mangle]
extern "C" fn c_close_file(handle: c_int) -> c_uint {
    debug!("c_close_file");
    match rtc_close_handle(handle) {
        Ok(size) => FFI_OK,
        Err(e) => {
            error!("c_close_file: {:?}", e);
            FFI_FILE_ERROR
        }
    }
}

/*
 * int teaclave_close_file(int fd);
 */
#[allow(unused)]
#[no_mangle]
pub extern "C" fn wasm_close_file(_exec_env: *const c_void, handle: c_int) -> c_int {
    debug!("wasm_close_file");
    match rtc_close_handle(handle) {
        Ok(size) => FFI_OK as i32,
        Err(e) => {
            error!("wasm_close_file: {:?}", e);
            FFI_FILE_ERROR_WASM
        }
    }
}
