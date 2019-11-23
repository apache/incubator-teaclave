// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Filesystem manipulation operations.

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::deps::sgx_aes_gcm_128bit_tag_t;
use crate::deps::sgx_key_128bit_t;
use crate::sgx_fs_inner as fs_imp;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// A reference to an open file on the filesystem.
///
/// An instance of a `File` can be read and/or written depending on what options
/// it was opened with. Files also implement [`Seek`] to alter the logical cursor
/// that the file contains internally.
///
/// Files are automatically closed when they go out of scope.
pub struct ProtectedFile {
    inner: fs_imp::SgxFile,
}

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a ProtectedFile is opened and
/// what operations are permitted on the open file. The ProtectedFile::open and
/// ProtectedFile::create methods are aliases for commonly used options using this
/// builder.
///
#[derive(Clone, Debug)]
pub struct OpenOptions(fs_imp::OpenOptions);

impl ProtectedFile {
    #[cfg(feature = "mesalock_sgx")]
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<ProtectedFile> {
        OpenOptions::default().read(true).open(path.as_ref())
    }

    #[cfg(feature = "mesalock_sgx")]
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<ProtectedFile> {
        OpenOptions::default().write(true).open(path.as_ref())
    }

    pub fn open_ex<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<ProtectedFile> {
        OpenOptions::default()
            .read(true)
            .open_ex(path.as_ref(), key)
    }

    pub fn create_ex<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<ProtectedFile> {
        OpenOptions::default()
            .write(true)
            .open_ex(path.as_ref(), key)
    }

    pub fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }

    pub fn clearerr(&self) {
        self.inner.clearerr()
    }

    pub fn clear_cache(&self) -> io::Result<()> {
        self.inner.clear_cache()
    }

    pub fn get_current_meta_gmac(
        &self,
        meta_gmac: &mut sgx_aes_gcm_128bit_tag_t,
    ) -> io::Result<()> {
        self.inner.get_current_meta_gmac(meta_gmac)
    }
}

impl Read for ProtectedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for ProtectedFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for ProtectedFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<'a> Read for &'a ProtectedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a> Write for &'a ProtectedFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<'a> Seek for &'a ProtectedFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    pub fn default() -> OpenOptions {
        OpenOptions(fs_imp::OpenOptions::new())
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.0.read(read);
        self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.0.write(write);
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// For most filesystems, the operating system guarantees that all writes are
    /// atomic: no writes get mangled because another process writes at the same
    /// time.
    ///
    /// One maybe obvious note when using append-mode: make sure that all data
    /// that belongs together is written to the file in one operation. This
    /// can be done by concatenating strings before passing them to `write()`,
    /// or using a buffered writer (with a buffer of adequate size),
    /// and calling `flush()` when the message is complete.
    ///
    /// If a file is opened with both read and append access, beware that after
    /// opening, and after every write, the position for reading may be set at the
    /// end of the file. So, before writing, save the current position (using
    /// `seek(SeekFrom::Current(0))`, and restore it before the next read.
    ///
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.0.append(append);
        self
    }

    /// Sets the option for update a previous file.
    pub fn update(&mut self, update: bool) -> &mut OpenOptions {
        self.0.update(update);
        self
    }

    /// Sets the option for binary a file.
    pub fn binary(&mut self, binary: bool) -> &mut OpenOptions {
        self.0.binary(binary);
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<ProtectedFile> {
        self._open(path.as_ref())
    }

    pub fn open_ex<P: AsRef<Path>>(
        &self,
        path: P,
        key: &sgx_key_128bit_t,
    ) -> io::Result<ProtectedFile> {
        self._open_ex(path.as_ref(), key)
    }

    fn _open(&self, path: &Path) -> io::Result<ProtectedFile> {
        let inner = fs_imp::SgxFile::open(path, &self.0)?;
        Ok(ProtectedFile { inner })
    }

    fn _open_ex(&self, path: &Path, key: &sgx_key_128bit_t) -> io::Result<ProtectedFile> {
        let inner = fs_imp::SgxFile::open_ex(path, &self.0, key)?;
        Ok(ProtectedFile { inner })
    }
}

pub fn remove_protected_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs_imp::remove(path.as_ref())
}

#[cfg(feature = "mesalock_sgx")]
pub fn export_auto_key<P: AsRef<Path>>(path: P) -> io::Result<sgx_key_128bit_t> {
    fs_imp::export_auto_key(path.as_ref())
}

#[cfg(feature = "mesalock_sgx")]
pub fn import_auto_key<P: AsRef<Path>>(path: P, key: &sgx_key_128bit_t) -> io::Result<()> {
    fs_imp::import_auto_key(path.as_ref(), key)
}

#[cfg(test)]
mod tests {
    use crate::remove_protected_file;
    use crate::ProtectedFile;
    use rand_core::RngCore;
    use std::io::{Read, Write};

    #[test]
    pub fn test_sgxfs() {
        let key = [90u8; 16];

        let mut write_data: [u8; 16] = [0; 16];
        let mut read_data: [u8; 16] = [0; 16];
        let write_size;
        let read_size;
        {
            let mut rng = rdrand::RdRand::new().unwrap();
            rng.fill_bytes(&mut write_data);

            let opt = ProtectedFile::create_ex("sgx_file", &key);
            assert_eq!(opt.is_ok(), true);
            let mut file = opt.unwrap();

            let result = file.write(&write_data);
            assert_eq!(result.is_ok(), true);
            write_size = result.unwrap();
        }

        {
            let opt = ProtectedFile::open_ex("sgx_file", &key);
            assert_eq!(opt.is_ok(), true);
            let mut file = opt.unwrap();

            let result = file.read(&mut read_data);
            assert_eq!(result.is_ok(), true);
            read_size = result.unwrap();
        }

        let result = remove_protected_file("sgx_file");
        assert_eq!(result.is_ok(), true);

        assert_eq!(write_data, read_data);
        assert_eq!(write_size, read_size);

        {
            let opt = ProtectedFile::open_ex("/", &key);
            assert_eq!(opt.is_err(), true);
            let opt = ProtectedFile::open_ex(".", &key);
            assert_eq!(opt.is_err(), true);
            let opt = ProtectedFile::open_ex("..", &key);
            assert_eq!(opt.is_err(), true);
            let opt = ProtectedFile::open_ex("?", &key);
            assert_eq!(opt.is_err(), true);
        }
        {
            let opt = ProtectedFile::create_ex("/", &key);
            assert_eq!(opt.is_err(), true);
        }
        {
            let opt = ProtectedFile::create_ex("/proc/100", &key);
            assert_eq!(opt.is_err(), true);
            let opt = ProtectedFile::create_ex(".", &key);
            assert_eq!(opt.is_err(), true);
            let opt = ProtectedFile::create_ex("..", &key);
            assert_eq!(opt.is_err(), true);
        }
    }
}
