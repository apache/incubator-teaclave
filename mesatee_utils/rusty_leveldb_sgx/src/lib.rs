//! rusty-leveldb is a reimplementation of LevelDB in pure rust. It depends only on a few crates,
//! and is very close to the original, implementation-wise. The external API is relatively small
//! and should be easy to use.
//!
//! ```
//! use rusty_leveldb::{DB, DBIterator, LdbIterator, Options};
//!
//! let opt = rusty_leveldb::in_memory();
//! let mut db = DB::open("mydatabase", opt).unwrap();
//!
//! db.put(b"Hello", b"World").unwrap();
//! assert_eq!(b"World", db.get(b"Hello").unwrap().as_slice());
//!
//! let mut iter = db.new_iter().unwrap();
//! // Note: For efficiency reasons, it's recommended to use advance() and current() instead of
//! // next() when iterating over many elements.
//! assert_eq!((b"Hello".to_vec(), b"World".to_vec()), iter.next().unwrap());
//!
//! db.delete(b"Hello").unwrap();
//! db.flush().unwrap();
//! ```
//!
#![cfg_attr(feature = "mesalock_sgx", no_std)]

#![allow(dead_code)]

#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate cfg_if;
cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        extern crate sgx_libc as libc;
        extern crate sgx_trts;
        extern crate sgx_types;
        extern crate protected_fs;
    } else {
        extern crate libc;
    }
}

extern crate crc;
extern crate integer_encoding;
extern crate rand;
extern crate snap;

#[cfg(test)]
#[macro_use]
extern crate time_test;

mod block;
mod block_builder;
mod blockhandle;
mod cache;
mod cmp;
mod disk_env;
mod env;
mod env_common;
mod error;
mod filter;
mod filter_block;
#[macro_use]
mod infolog;
mod key_types;
mod log;
mod mem_env;
mod memtable;
mod merging_iter;
mod options;
mod skipmap;
mod snapshot;
mod table_block;
mod table_builder;
mod table_cache;
mod table_reader;
mod test_util;
mod types;
mod version;
mod version_edit;
mod version_set;
mod write_batch;

mod db_impl;
mod db_iter;

pub use cmp::{Cmp, DefaultCmp};
pub use db_impl::DB;
pub use db_iter::DBIterator;
pub use disk_env::PosixDiskEnv;
pub use env::Env;
pub use error::{Result, Status, StatusCode};
pub use filter::{BloomPolicy, FilterPolicy};
pub use mem_env::MemEnv;
pub use options::{in_memory, CompressionType, Options};
pub use skipmap::SkipMap;
pub use types::LdbIterator;
pub use write_batch::WriteBatch;
