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
#![allow(dead_code)]
#![allow(clippy::all)]

extern crate protected_fs;
extern crate sgx_libc as libc;
extern crate sgx_trts;
extern crate sgx_types;

extern crate crc;
extern crate integer_encoding;
extern crate rand;
extern crate snap;

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

pub use crate::cmp::{Cmp, DefaultCmp};
pub use crate::db_iter::DBIterator;
pub use crate::env::Env;
pub use crate::error::{Result, Status, StatusCode};
pub use crate::filter::{BloomPolicy, FilterPolicy};
pub use crate::mem_env::MemEnv;
pub use crate::options::{in_memory, CompressionType, Options};
pub use crate::skipmap::SkipMap;
pub use crate::types::LdbIterator;
pub use crate::write_batch::WriteBatch;
pub use db_impl::DB;
pub use disk_env::PosixDiskEnv;

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::check_all_passed;

    pub fn run_tests() -> bool {
        check_all_passed!(
            block::tests::run_tests(),
            block_builder::tests::run_tests(),
            blockhandle::tests::run_tests(),
            cache::tests::run_tests(),
            cmp::tests::run_tests(),
            db_impl::tests::run_tests(),
            db_iter::tests::run_tests(),
            disk_env::tests::run_tests(),
            error::tests::run_tests(),
            filter::tests::run_tests(),
            filter_block::tests::run_tests(),
            key_types::tests::run_tests(),
            log::tests::run_tests(),
            mem_env::tests::run_tests(),
            memtable::tests::run_tests(),
            merging_iter::tests::run_tests(),
            skipmap::tests::run_tests(),
            snapshot::tests::run_tests(),
            table_builder::tests::run_tests(),
            table_cache::tests::run_tests(),
            test_util::tests::run_tests(),
            table_reader::tests::run_tests(),
            types::tests::run_tests(),
            version::tests::run_tests(),
            version_edit::tests::run_tests(),
            version_set::tests::run_tests(),
            write_batch::tests::run_tests(),
        )
    }
}
