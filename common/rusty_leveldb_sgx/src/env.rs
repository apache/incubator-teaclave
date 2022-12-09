//! An `env` is an abstraction layer that allows the database to run both on different platforms as
//! well as persisting data on disk or in memory.

use crate::error::Result;

use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

use sgx_tprotected_fs::SgxFile;

pub trait RandomAccess {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize>;
}

impl RandomAccess for SgxFile {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        Ok((self as &dyn FileExt).read_at(dst, off as u64)?)
    }
}

pub struct FileLock {
    pub id: String,
}

pub trait Env {
    fn open_sequential_file(&self, p: &Path) -> Result<Box<dyn Read>>;
    fn open_random_access_file(&self, p: &Path) -> Result<Box<dyn RandomAccess>>;
    fn open_writable_file(&self, p: &Path) -> Result<Box<dyn Write>>;
    fn open_appendable_file(&self, p: &Path) -> Result<Box<dyn Write>>;

    fn exists(&self, p: &Path) -> Result<bool>;
    fn children(&self, p: &Path) -> Result<Vec<PathBuf>>;
    fn size_of(&self, p: &Path) -> Result<usize>;

    fn delete(&self, p: &Path) -> Result<()>;
    fn mkdir(&self, p: &Path) -> Result<()>;
    fn rmdir(&self, p: &Path) -> Result<()>;
    fn rename(&self, p: &Path, p: &Path) -> Result<()>;

    fn lock(&self, p: &Path) -> Result<FileLock>;
    fn unlock(&self, l: FileLock) -> Result<()>;

    fn new_logger(&self, p: &Path) -> Result<Logger>;

    fn micros(&self) -> u64;
}

pub struct Logger {
    dst: Box<dyn Write>,
}

impl Logger {
    pub fn new(w: Box<dyn Write>) -> Logger {
        Logger { dst: w }
    }

    pub fn log(&mut self, message: &str) {
        let _ = self.dst.write(message.as_bytes());
        let _ = self.dst.write("\n".as_bytes());
    }
}

pub fn path_to_string(p: &Path) -> String {
    p.to_str().map(String::from).unwrap()
}

pub fn path_to_str(p: &Path) -> &str {
    p.to_str().unwrap()
}
