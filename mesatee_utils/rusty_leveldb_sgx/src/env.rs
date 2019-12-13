//! An `env` is an abstraction layer that allows the database to run both on different platforms as
//! well as persisting data on disk or in memory.
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use error::Result;

use std::io::prelude::*;
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        use protected_fs::ProtectedFile;
        use error::Status;
        use std::untrusted::fs::File;
    } else {
        use std::fs::File;
    }
}

pub trait RandomAccess {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize>;
}

impl RandomAccess for File {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        Ok((self as &dyn FileExt).read_at(dst, off as u64)?)
    }
}

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        impl RandomAccess for ProtectedFile {
            fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
                self.read_at(off, dst).map_err(|e| Status::from(e))
            }
        }
    }
}

pub struct FileLock {
    pub id: String,
}

pub trait Env {
    fn open_sequential_file(&self, &Path) -> Result<Box<dyn Read>>;
    fn open_random_access_file(&self, &Path) -> Result<Box<dyn RandomAccess>>;
    fn open_writable_file(&self, &Path) -> Result<Box<dyn Write>>;
    fn open_appendable_file(&self, &Path) -> Result<Box<dyn Write>>;

    fn exists(&self, &Path) -> Result<bool>;
    fn children(&self, &Path) -> Result<Vec<PathBuf>>;
    fn size_of(&self, &Path) -> Result<usize>;

    fn delete(&self, &Path) -> Result<()>;
    fn mkdir(&self, &Path) -> Result<()>;
    fn rmdir(&self, &Path) -> Result<()>;
    fn rename(&self, &Path, &Path) -> Result<()>;

    fn lock(&self, &Path) -> Result<FileLock>;
    fn unlock(&self, l: FileLock) -> Result<()>;

    fn new_logger(&self, &Path) -> Result<Logger>;

    fn micros(&self) -> u64;
}

pub struct Logger {
    dst: Box<dyn Write>,
}

impl Logger {
    pub fn new(w: Box<dyn Write>) -> Logger {
        Logger { dst: w }
    }

    pub fn log(&mut self, message: &String) {
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
