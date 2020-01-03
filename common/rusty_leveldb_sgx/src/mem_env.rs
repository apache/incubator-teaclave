//! An in-memory implementation of Env.
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use env::{path_to_str, path_to_string, Env, FileLock, Logger, RandomAccess};
use env_common::micros;
use error::{err, Result, StatusCode};

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        use std::sync::{Arc, SgxMutex as Mutex};
    } else {
        use std::sync::{Arc, Mutex};
    }
}

/// BufferBackedFile is a simple type implementing RandomAccess on a Vec<u8>.
pub type BufferBackedFile = Vec<u8>;

impl RandomAccess for BufferBackedFile {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        if off > self.len() {
            return Ok(0);
        }
        let remaining = self.len() - off;
        let to_read = if dst.len() > remaining {
            remaining
        } else {
            dst.len()
        };
        (&mut dst[0..to_read]).copy_from_slice(&self[off..off + to_read]);
        Ok(to_read)
    }
}

/// A MemFile holds a shared, concurrency-safe buffer. It can be shared among several
/// MemFileReaders and MemFileWriters, each with an independent offset.
#[derive(Clone)]
pub struct MemFile(Arc<Mutex<BufferBackedFile>>);

impl MemFile {
    fn new() -> MemFile {
        MemFile(Arc::new(Mutex::new(Vec::new())))
    }
}

/// A MemFileReader holds a reference to a MemFile and a read offset.
struct MemFileReader(MemFile, usize);

impl MemFileReader {
    fn new(f: MemFile, from: usize) -> MemFileReader {
        MemFileReader(f, from)
    }
}

// We need Read/Write/Seek implementations for our MemFile in order to work well with the
// concurrency requirements. It's very hard or even impossible to implement those traits just by
// wrapping MemFile in other types.
impl Read for MemFileReader {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        let buf = (self.0).0.lock().unwrap();
        if self.1 >= buf.len() {
            // EOF
            return Ok(0);
        }
        let remaining = buf.len() - self.1;
        let to_read = if dst.len() > remaining {
            remaining
        } else {
            dst.len()
        };

        (&mut dst[0..to_read]).copy_from_slice(&buf[self.1..self.1 + to_read]);
        self.1 += to_read;
        Ok(to_read)
    }
}

/// A MemFileWriter holds a reference to a MemFile and a write offset.
struct MemFileWriter(MemFile, usize);

impl MemFileWriter {
    fn new(f: MemFile, append: bool) -> MemFileWriter {
        let len = f.0.lock().unwrap().len();
        MemFileWriter(f, if append { len } else { 0 })
    }
}

impl Write for MemFileWriter {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        let mut buf = (self.0).0.lock().unwrap();
        // Write is append.
        if self.1 == buf.len() {
            buf.extend_from_slice(src);
        } else {
            // Write in the middle, possibly appending.
            let remaining = buf.len() - self.1;
            if src.len() <= remaining {
                // src fits into buffer.
                (&mut buf[self.1..self.1 + src.len()]).copy_from_slice(src);
            } else {
                // src doesn't fit; first copy what fits, then append the rest/
                (&mut buf[self.1..self.1 + remaining]).copy_from_slice(&src[0..remaining]);
                buf.extend_from_slice(&src[remaining..src.len()]);
            }
        }
        self.1 += src.len();
        Ok(src.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl RandomAccess for MemFile {
    fn read_at(&self, off: usize, dst: &mut [u8]) -> Result<usize> {
        let guard = self.0.lock().unwrap();
        let buf: &BufferBackedFile = guard.deref();
        buf.read_at(off, dst)
    }
}

struct MemFSEntry {
    f: MemFile,
    locked: bool,
}

/// MemFS implements a completely in-memory file system, both for testing and temporary in-memory
/// databases. It supports full concurrency.
pub struct MemFS {
    store: Arc<Mutex<HashMap<String, MemFSEntry>>>,
}

impl MemFS {
    fn new() -> MemFS {
        MemFS {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Open a file. The caller can use the MemFile either inside a MemFileReader or as
    /// RandomAccess.
    fn open(&self, p: &Path, create: bool) -> Result<MemFile> {
        let mut fs = self.store.lock().unwrap();
        match fs.entry(path_to_string(p)) {
            Entry::Occupied(o) => Ok(o.get().f.clone()),
            Entry::Vacant(v) => {
                if !create {
                    return err(
                        StatusCode::NotFound,
                        &format!("open: file not found: {}", path_to_str(p)),
                    );
                }
                let f = MemFile::new();
                v.insert(MemFSEntry {
                    f: f.clone(),
                    locked: false,
                });
                Ok(f)
            }
        }
    }
    /// Open a file for writing.
    fn open_w(&self, p: &Path, append: bool, truncate: bool) -> Result<Box<dyn Write>> {
        let f = self.open(p, true)?;
        if truncate {
            f.0.lock().unwrap().clear();
        }
        Ok(Box::new(MemFileWriter::new(f, append)))
    }
    fn exists_(&self, p: &Path) -> Result<bool> {
        let fs = self.store.lock()?;
        Ok(fs.contains_key(path_to_str(p)))
    }
    fn children_of(&self, p: &Path) -> Result<Vec<PathBuf>> {
        let fs = self.store.lock()?;
        let mut prefix = path_to_string(p);
        if !prefix.ends_with("/") {
            prefix.push('/');
        }
        let mut children = Vec::new();
        for k in fs.keys() {
            if k.starts_with(&prefix) {
                children.push(Path::new(k.trim_start_matches(&prefix)).to_owned());
            }
        }
        Ok(children)
    }
    fn size_of_(&self, p: &Path) -> Result<usize> {
        let mut fs = self.store.lock()?;
        match fs.entry(path_to_string(p)) {
            Entry::Occupied(o) => Ok(o.get().f.0.lock()?.len()),
            _ => err(
                StatusCode::NotFound,
                &format!("size_of: file not found: {}", path_to_str(p)),
            ),
        }
    }
    fn delete_(&self, p: &Path) -> Result<()> {
        let mut fs = self.store.lock()?;
        match fs.entry(path_to_string(p)) {
            Entry::Occupied(o) => {
                o.remove_entry();
                Ok(())
            }
            _ => err(
                StatusCode::NotFound,
                &format!("delete: file not found: {}", path_to_str(p)),
            ),
        }
    }
    fn rename_(&self, from: &Path, to: &Path) -> Result<()> {
        let mut fs = self.store.lock()?;
        match fs.remove(path_to_str(from)) {
            Some(v) => {
                fs.insert(path_to_string(to), v);
                Ok(())
            }
            _ => err(
                StatusCode::NotFound,
                &format!("rename: file not found: {}", path_to_str(from)),
            ),
        }
    }
    fn lock_(&self, p: &Path) -> Result<FileLock> {
        let mut fs = self.store.lock()?;
        match fs.entry(path_to_string(p)) {
            Entry::Occupied(mut o) => {
                if o.get().locked {
                    err(
                        StatusCode::LockError,
                        &format!("already locked: {}", path_to_str(p)),
                    )
                } else {
                    o.get_mut().locked = true;
                    Ok(FileLock {
                        id: path_to_string(p),
                    })
                }
            }
            Entry::Vacant(v) => {
                let f = MemFile::new();
                v.insert(MemFSEntry {
                    f: f.clone(),
                    locked: true,
                });
                Ok(FileLock {
                    id: path_to_string(p),
                })
            }
        }
    }
    fn unlock_(&self, l: FileLock) -> Result<()> {
        let mut fs = self.store.lock()?;
        let id = l.id.clone();
        match fs.entry(l.id) {
            Entry::Occupied(mut o) => {
                if !o.get().locked {
                    err(
                        StatusCode::LockError,
                        &format!("unlocking unlocked file: {}", id),
                    )
                } else {
                    o.get_mut().locked = false;
                    Ok(())
                }
            }
            _ => err(
                StatusCode::NotFound,
                &format!("unlock: file not found: {}", id),
            ),
        }
    }
}

/// MemEnv is an in-memory environment that can be used for testing or ephemeral databases. The
/// performance will be better than what a disk environment delivers.
pub struct MemEnv(MemFS);

impl MemEnv {
    pub fn new() -> MemEnv {
        MemEnv(MemFS::new())
    }
}

impl Env for MemEnv {
    fn open_sequential_file(&self, p: &Path) -> Result<Box<dyn Read>> {
        let f = self.0.open(p, false)?;
        Ok(Box::new(MemFileReader::new(f, 0)))
    }
    fn open_random_access_file(&self, p: &Path) -> Result<Box<dyn RandomAccess>> {
        self.0
            .open(p, false)
            .map(|m| Box::new(m) as Box<dyn RandomAccess>)
    }
    fn open_writable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_w(p, true, true)
    }
    fn open_appendable_file(&self, p: &Path) -> Result<Box<dyn Write>> {
        self.0.open_w(p, true, false)
    }

    fn exists(&self, p: &Path) -> Result<bool> {
        self.0.exists_(p)
    }
    fn children(&self, p: &Path) -> Result<Vec<PathBuf>> {
        self.0.children_of(p)
    }
    fn size_of(&self, p: &Path) -> Result<usize> {
        self.0.size_of_(p)
    }

    fn delete(&self, p: &Path) -> Result<()> {
        self.0.delete_(p)
    }
    fn mkdir(&self, p: &Path) -> Result<()> {
        if self.exists(p)? {
            err(StatusCode::AlreadyExists, "")
        } else {
            Ok(())
        }
    }
    fn rmdir(&self, p: &Path) -> Result<()> {
        if !self.exists(p)? {
            err(StatusCode::NotFound, "")
        } else {
            Ok(())
        }
    }
    fn rename(&self, old: &Path, new: &Path) -> Result<()> {
        self.0.rename_(old, new)
    }

    fn lock(&self, p: &Path) -> Result<FileLock> {
        self.0.lock_(p)
    }
    fn unlock(&self, p: FileLock) -> Result<()> {
        self.0.unlock_(p)
    }

    fn micros(&self) -> u64 {
        micros()
    }

    fn new_logger(&self, p: &Path) -> Result<Logger> {
        self.open_appendable_file(p)
            .map(|dst| Logger::new(Box::new(dst)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env;

    fn new_memfile(v: Vec<u8>) -> MemFile {
        MemFile(Arc::new(Mutex::new(v)))
    }

    #[test]
    fn test_mem_fs_memfile_read() {
        let f = new_memfile(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let mut buf: [u8; 1] = [0];
        let mut reader = MemFileReader(f, 0);

        for i in [1, 2, 3, 4, 5, 6, 7, 8].iter() {
            assert_eq!(reader.read(&mut buf).unwrap(), 1);
            assert_eq!(buf, [*i]);
        }
    }

    #[test]
    fn test_mem_fs_memfile_write() {
        let f = new_memfile(vec![]);
        let mut w1 = MemFileWriter::new(f.clone(), false);
        assert_eq!(w1.write(&[1, 2, 3]).unwrap(), 3);

        let mut w2 = MemFileWriter::new(f, true);
        assert_eq!(w1.write(&[1, 7, 8, 9]).unwrap(), 4);
        assert_eq!(w2.write(&[4, 5, 6]).unwrap(), 3);

        assert_eq!(
            (w1.0).0.lock().unwrap().as_ref() as &Vec<u8>,
            &[1, 2, 3, 4, 5, 6, 9]
        );
    }

    #[test]
    fn test_mem_fs_memfile_readat() {
        let f = new_memfile(vec![1, 2, 3, 4, 5]);

        let mut buf = [0; 3];
        assert_eq!(f.read_at(2, &mut buf).unwrap(), 3);
        assert_eq!(buf, [3, 4, 5]);

        assert_eq!(f.read_at(0, &mut buf[0..1]).unwrap(), 1);
        assert_eq!(buf, [1, 4, 5]);

        assert_eq!(f.read_at(5, &mut buf).unwrap(), 0);
        assert_eq!(buf, [1, 4, 5]);

        let mut buf2 = [0; 6];
        assert_eq!(f.read_at(0, &mut buf2[0..5]).unwrap(), 5);
        assert_eq!(buf2, [1, 2, 3, 4, 5, 0]);
        assert_eq!(f.read_at(0, &mut buf2[0..6]).unwrap(), 5);
        assert_eq!(buf2, [1, 2, 3, 4, 5, 0]);
    }

    #[test]
    fn test_mem_fs_open_read_write() {
        let fs = MemFS::new();
        let path = Path::new("/a/b/hello.txt");

        {
            let mut w = fs.open_w(&path, false, false).unwrap();
            write!(w, "Hello").unwrap();
            // Append.
            let mut w2 = fs.open_w(&path, true, false).unwrap();
            write!(w2, "World").unwrap();
        }
        {
            let mut r = MemFileReader::new(fs.open(&path, false).unwrap(), 0);
            let mut s = String::new();
            assert_eq!(r.read_to_string(&mut s).unwrap(), 10);
            assert_eq!(s, "HelloWorld");

            let mut r2 = MemFileReader::new(fs.open(&path, false).unwrap(), 2);
            s.clear();
            assert_eq!(r2.read_to_string(&mut s).unwrap(), 8);
            assert_eq!(s, "lloWorld");
        }
        assert_eq!(fs.size_of_(&path).unwrap(), 10);
        assert!(fs.exists_(&path).unwrap());
        assert!(!fs.exists_(&Path::new("/non/existing/path")).unwrap());
    }

    #[test]
    fn test_mem_fs_open_read_write_append_truncate() {
        let fs = MemFS::new();
        let path = Path::new("/a/b/hello.txt");

        {
            let mut w0 = fs.open_w(&path, false, true).unwrap();
            write!(w0, "Garbage").unwrap();

            // Truncate.
            let mut w = fs.open_w(&path, false, true).unwrap();
            write!(w, "Xyz").unwrap();
            // Write to the beginning.
            let mut w2 = fs.open_w(&path, false, false).unwrap();
            write!(w2, "OverwritingEverythingWithGarbage").unwrap();
            // Overwrite the overwritten stuff.
            write!(w, "Xyz").unwrap();
            assert!(w.flush().is_ok());
        }
        {
            let mut r = MemFileReader::new(fs.open(&path, false).unwrap(), 0);
            let mut s = String::new();
            assert_eq!(r.read_to_string(&mut s).unwrap(), 32);
            assert_eq!(s, "OveXyzitingEverythingWithGarbage");
        }
        assert!(fs.exists_(&path).unwrap());
        assert_eq!(fs.size_of_(&path).unwrap(), 32);
        assert!(!fs.exists_(&Path::new("/non/existing/path")).unwrap());
    }

    #[test]
    fn test_mem_fs_metadata_operations() {
        let fs = MemFS::new();
        let path = Path::new("/a/b/hello.file");
        let newpath = Path::new("/a/b/hello2.file");
        let nonexist = Path::new("/blah");

        // Make file/remove file.
        {
            let mut w = fs.open_w(&path, false, false).unwrap();
            write!(w, "Hello").unwrap();
        }
        assert!(fs.exists_(&path).unwrap());
        assert_eq!(fs.size_of_(&path).unwrap(), 5);
        fs.delete_(&path).unwrap();
        assert!(!fs.exists_(&path).unwrap());
        assert!(fs.delete_(&nonexist).is_err());

        // rename_ file.
        {
            let mut w = fs.open_w(&path, false, false).unwrap();
            write!(w, "Hello").unwrap();
        }
        assert!(fs.exists_(&path).unwrap());
        assert!(!fs.exists_(&newpath).unwrap());
        assert_eq!(fs.size_of_(&path).unwrap(), 5);
        assert!(fs.size_of_(&newpath).is_err());

        fs.rename_(&path, &newpath).unwrap();

        assert!(!fs.exists_(&path).unwrap());
        assert!(fs.exists_(&newpath).unwrap());
        assert_eq!(fs.size_of_(&newpath).unwrap(), 5);
        assert!(fs.size_of_(&path).is_err());

        assert!(fs.rename_(&nonexist, &path).is_err());
    }

    fn s2p(x: &str) -> PathBuf {
        Path::new(x).to_owned()
    }

    #[test]
    fn test_mem_fs_children() {
        let fs = MemFS::new();
        let (path1, path2, path3) = (
            Path::new("/a/1.txt"),
            Path::new("/a/2.txt"),
            Path::new("/b/1.txt"),
        );

        for p in &[&path1, &path2, &path3] {
            fs.open_w(*p, false, false).unwrap();
        }
        let children = fs.children_of(&Path::new("/a")).unwrap();
        assert!(
            (children == vec![s2p("1.txt"), s2p("2.txt")])
                || (children == vec![s2p("2.txt"), s2p("1.txt")])
        );
        let children = fs.children_of(&Path::new("/a/")).unwrap();
        assert!(
            (children == vec![s2p("1.txt"), s2p("2.txt")])
                || (children == vec![s2p("2.txt"), s2p("1.txt")])
        );
    }

    #[test]
    fn test_mem_fs_lock() {
        let fs = MemFS::new();
        let p = Path::new("/a/lock");

        {
            let mut f = fs.open_w(p, true, true).unwrap();
            f.write("abcdef".as_bytes()).expect("write failed");
        }

        // Locking on new file.
        let lock = fs.lock_(p).unwrap();
        assert!(fs.lock_(p).is_err());

        // Unlock of locked file is ok.
        assert!(fs.unlock_(lock).is_ok());

        // Lock of unlocked file is ok.
        let lock = fs.lock_(p).unwrap();
        assert!(fs.lock_(p).is_err());
        assert!(fs.unlock_(lock).is_ok());

        // Rogue operation.
        assert!(fs
            .unlock_(env::FileLock {
                id: "/a/lock".to_string(),
            })
            .is_err());

        // Non-existent files.
        let p2 = Path::new("/a/lock2");
        assert!(fs.lock_(p2).is_ok());
        assert!(fs
            .unlock_(env::FileLock {
                id: "/a/lock2".to_string(),
            })
            .is_ok());
    }

    #[test]
    fn test_memenv_all() {
        let me = MemEnv::new();
        let (p1, p2, p3) = (Path::new("/a/b"), Path::new("/a/c"), Path::new("/a/d"));
        let nonexist = Path::new("/x/y");
        me.open_writable_file(p2).unwrap();
        me.open_appendable_file(p3).unwrap();
        me.open_sequential_file(p2).unwrap();
        me.open_random_access_file(p3).unwrap();

        assert!(me.exists(p2).unwrap());
        assert_eq!(me.children(Path::new("/a/")).unwrap().len(), 2);
        assert_eq!(me.size_of(p2).unwrap(), 0);

        me.delete(p2).unwrap();
        assert!(me.mkdir(p3).is_err());
        me.mkdir(p1).unwrap();
        me.rmdir(p3).unwrap();
        assert!(me.rmdir(nonexist).is_err());

        me.open_writable_file(p1).unwrap();
        me.rename(p1, p3).unwrap();
        assert!(!me.exists(p1).unwrap());
        assert!(me.rename(nonexist, p1).is_err());

        me.unlock(me.lock(p3).unwrap()).unwrap();
        assert!(me.lock(nonexist).is_ok());

        me.new_logger(p1).unwrap();
        assert!(me.micros() > 0);
    }
}
