/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::ffi::CString;
use std::io;
use std::os::raw::{c_char, c_int};

enum TeaclaveContextFilePermission {
    Read,
    Write,
}

/// This struct is a wrapped version of the `teaclave_*` C interfaces
///  and meant to provide a set of more convenient methods for Rust  
/// `std::io::Read` and `std::io::Write` traits are implemented
///  for TeaclaveContextFile to support related traits  
/// The protected file handlers need NOT to be closed manually
pub struct TeaclaveContextFile {
    handle: i32,
    permission: TeaclaveContextFilePermission,
}

pub struct TeaclaveContextFileError;

type Result<T> = std::result::Result<T, io::Error>;

impl io::Read for TeaclaveContextFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.permission {
            TeaclaveContextFilePermission::Write => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Can't read opened input",
            )),
            _ => {
                let rv =
                    unsafe { teaclave_read_file(self.handle, buf.as_ptr() as _, buf.len() as _) };
                if rv == -1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "teaclave_read_file failed",
                    ));
                }
                Ok(rv as _)
            }
        }
    }
}

impl io::Write for TeaclaveContextFile {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.permission {
            TeaclaveContextFilePermission::Read => Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Can't read opened input",
            )),
            _ => {
                let rv =
                    unsafe { teaclave_write_file(self.handle, buf.as_ptr() as _, buf.len() as _) };
                if rv == -1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "teaclave_write_file failed",
                    ));
                }
                Ok(rv as _)
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        return Ok(());
    }
}

impl TeaclaveContextFile {
    /// A wrapped version of `teaclave_open_input`
    pub fn open_input(fid: &str) -> Result<Self> {
        let fid_owned = CString::new(fid).unwrap();
        let fd = unsafe { teaclave_open_input(fid_owned.as_c_str().as_ptr() as _) };
        if fd == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "teaclave_open_input failed",
            ));
        }
        Ok(TeaclaveContextFile {
            handle: fd,
            permission: TeaclaveContextFilePermission::Read,
        })
    }

    /// A wrapped version of `teaclave_create_output`
    pub fn create_output(fid: &str) -> Result<Self> {
        let fid_owned = CString::new(fid).unwrap();
        let fd = unsafe { teaclave_create_output(fid_owned.as_c_str().as_ptr() as _) };
        if fd == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "teaclave_create_output failed",
            ));
        }
        Ok(TeaclaveContextFile {
            handle: fd,
            permission: TeaclaveContextFilePermission::Write,
        })
    }
}

impl std::ops::Drop for TeaclaveContextFile {
    fn drop(&mut self) {
        unsafe { teaclave_close_file(self.handle) };
    }
}

extern "C" {
    /// Close a file handler
    ///
    /// # Arguments
    ///
    /// * `fd` - file handler returned by `teaclave_open_input`
    ///
    /// # Return
    ///
    /// 0 if succeed, -1 otherwise
    pub fn teaclave_close_file(fd: c_int) -> c_int;

    /// Write content from a buffer to a file
    ///
    /// # Arguments
    ///
    /// * `fd` - file handler returned by `teaclave_open_input`
    /// * `in_buf` - the pointer to the buffer holding content to write
    /// * `buf_size` - the total size in bytes to read from the buffer and write to the file
    ///
    /// # Return
    ///
    /// bytes written to the file, -1 if error occurs
    pub fn teaclave_write_file(fd: c_int, in_buf: *mut c_char, buf_size: c_int) -> c_int;

    /// Read content from a file to a buffer
    ///
    /// # Arguments
    ///
    /// * `fd` - file handler returned by `teaclave_open_input`
    /// * `out_buf` - the pointer to output buffer
    /// * `buf_size` - the total size in bytes of the output buffer
    ///
    /// # Return
    ///
    /// bytes read from the file, -1 if error occurs
    pub fn teaclave_read_file(fd: c_int, out_buf: *mut c_char, buf_size: c_int) -> c_int;

    /// Create or open a protected file as output
    ///
    /// # Arguments
    ///
    /// * `fid` - the uid of the file, c string pointer
    ///
    /// # Return
    ///
    /// file handler, -1 if error occurs
    pub fn teaclave_create_output(fid: *mut c_char) -> c_int;

    /// Open a protected file as input
    ///
    /// # Arguments
    ///
    /// * `fid` - the uid of the file, c string pointer
    ///
    /// # Return
    ///
    /// file handler, -1 if error occurs
    pub fn teaclave_open_input(fid: *mut c_char) -> c_int;

}
