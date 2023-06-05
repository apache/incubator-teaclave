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

// This file references
// https://github.com/quickwit-oss/tantivy/blob/main/src/directory/ram_directory.rs

use teaclave_proto::teaclave_storage_service::{
    DeleteRequest, GetRequest, PutRequest, TeaclaveStorageClient,
};
use teaclave_rpc::transport::Channel;

use std::io::{self, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::{fmt, result};

use tantivy::directory::error::{DeleteError, OpenReadError, OpenWriteError};
use tantivy::directory::{
    AntiCallToken, Directory, FileHandle, FileSlice, TerminatingWrite, WatchCallback,
    WatchCallbackList, WatchHandle, WritePtr,
};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::Mutex;

pub static META_FILEPATH: LazyLock<&'static Path> = LazyLock::new(|| Path::new("meta.json"));
pub static DB_PREFIX: LazyLock<String> = LazyLock::new(|| String::from("tantivy/"));
static INDEX_WRITER_LOCK: LazyLock<&'static Path> =
    LazyLock::new(|| Path::new(".tantivy-writer.lock"));

struct Cache {
    path: PathBuf,
    shared_directory: DbDirectory,
    data: Cursor<Vec<u8>>,
    is_flushed: bool,
}

impl Cache {
    fn new(path_buf: PathBuf, shared_directory: DbDirectory) -> Self {
        Cache {
            path: path_buf,
            data: Cursor::new(Vec::new()),
            shared_directory,
            is_flushed: true,
        }
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.is_flushed {
            let _ = self.flush();
        }
    }
}

impl Seek for Cache {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.data.seek(pos)
    }
}

impl Write for Cache {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.is_flushed = false;
        self.data.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.shared_directory
            .write(&self.path, self.data.get_ref())?;
        self.is_flushed = true;

        Ok(())
    }
}

impl TerminatingWrite for Cache {
    fn terminate_ref(&mut self, _: AntiCallToken) -> io::Result<()> {
        self.flush()
    }
}

/// A Directory storing everything in the storage service.
#[derive(Clone)]
pub struct DbDirectory {
    db: Arc<Mutex<TeaclaveStorageClient<Channel>>>,
    watch_router: Arc<WatchCallbackList>,
    rt: Arc<Runtime>,
}

impl fmt::Debug for DbDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DbDirectory")
    }
}

impl DbDirectory {
    pub fn new(db: Arc<Mutex<TeaclaveStorageClient<Channel>>>) -> Self {
        let rt = Arc::new(Builder::new_current_thread().enable_all().build().unwrap());
        let dir = Self {
            db,
            watch_router: Arc::default(),
            rt,
        };

        // remove the lockfile if it exists
        let _ = dir.delete(&INDEX_WRITER_LOCK);

        dir
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        let key = DB_PREFIX.clone() + &path.to_string_lossy();
        let request = PutRequest::new(key.as_bytes(), data);

        self.rt
            .block_on(self.db.blocking_lock().put(request))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }
}

impl Directory for DbDirectory {
    fn get_file_handle(&self, path: &Path) -> Result<Arc<dyn FileHandle>, OpenReadError> {
        let file_slice = self.open_read(path)?;
        Ok(Arc::new(file_slice))
    }

    fn open_read(&self, path: &Path) -> result::Result<FileSlice, OpenReadError> {
        let key = DB_PREFIX.clone() + &path.to_string_lossy();
        let request = GetRequest::new(key.as_bytes());

        self.rt
            .block_on(self.db.blocking_lock().get(request))
            .map_err(|_| OpenReadError::FileDoesNotExist(PathBuf::from(path)))
            .map(|r| FileSlice::from(r.into_inner().value))
    }

    fn delete(&self, path: &Path) -> result::Result<(), DeleteError> {
        let key = DB_PREFIX.clone() + &path.to_string_lossy();
        let request = DeleteRequest::new(key.as_bytes());

        self.rt
            .block_on(self.db.blocking_lock().delete(request))
            .map_err(|_| DeleteError::FileDoesNotExist(PathBuf::from(path)))?;
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool, OpenReadError> {
        let key = DB_PREFIX.clone() + &path.to_string_lossy();
        let request = GetRequest::new(key.as_bytes());

        let get = self.rt.block_on(self.db.blocking_lock().get(request));
        Ok(get.is_ok())
    }

    fn open_write(&self, path: &Path) -> Result<WritePtr, OpenWriteError> {
        let cache = Cache::new(path.to_owned(), self.clone());
        let exists = self.exists(path).unwrap();
        // force the creation of the file to mimic the MMap directory.
        if exists {
            self.write(path, &[])
                .map_err(|io_error| OpenWriteError::IoError {
                    io_error: Arc::new(io_error),
                    filepath: PathBuf::from(path),
                })?;

            Err(OpenWriteError::FileAlreadyExists(PathBuf::from(path)))
        } else {
            Ok(BufWriter::new(Box::new(cache)))
        }
    }

    fn atomic_read(&self, path: &Path) -> Result<Vec<u8>, OpenReadError> {
        let bytes =
            self.open_read(path)?
                .read_bytes()
                .map_err(|io_error| OpenReadError::IoError {
                    io_error: Arc::new(io_error),
                    filepath: PathBuf::from(path),
                })?;
        Ok(bytes.as_slice().to_owned())
    }

    fn atomic_write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.write(path, data)?;
        if path == *META_FILEPATH {
            drop(self.watch_router.broadcast());
        }
        Ok(())
    }

    fn watch(&self, watch_callback: WatchCallback) -> tantivy::Result<WatchHandle> {
        Ok(self.watch_router.subscribe(watch_callback))
    }

    fn sync_directory(&self) -> io::Result<()> {
        Ok(())
    }
}
