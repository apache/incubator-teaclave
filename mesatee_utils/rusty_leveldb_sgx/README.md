# leveldb-rs
[![Build Status](https://ci.mesalock-linux.org/api/badges/mesalock-linux/rusty_leveldb_sgx/status.svg)](https://ci.mesalock-linux.org/mesalock-linux/rusty_leveldb_sgx)
[![crates.io](https://img.shields.io/crates/v/rusty-leveldb.svg)](https://crates.io/crates/rusty-leveldb)

A fully compatible implementation of LevelDB in Rust. (any incompatibility is a
bug!)

The implementation is very close to the original; often, you can see the same
algorithm translated 1:1, and class (struct) and method names are similar or
the same.

**NOTE: I do not endorse using this library for any data that you care about.**
I do care, however, about bug reports.

## Status

* User-facing methods exist: Read/Write/Delete; snapshots; iteration
* Compaction is supported, including manual ones.
* Fully synchronous: Efficiency gains by using non-atomic types, but writes may
  occasionally block during a compaction. In --release mode, an average compaction
  takes 0.2-0.5 seconds.
* Compatibility with the original: Compression is not implemented so far; this works
  as long as compression is disabled in the original.
* Performance is decent; while usually not par with the original, due to multi-threading
  in the original and language-inherent overhead (we are doing things the right way),
  it will be enough for most use cases.
* Safe: While using many shared pointers, the implementation is generally safe. Many
  places use asserts though, so you may see a crash -- in which case you should file a bug.

## Goals

Some of the goals of this implementation are

* As few copies of data as possible; most of the time, slices of bytes (`&[u8]`)
  are used. Owned memory is represented as `Vec<u8>` (and then possibly borrowed
  as slice).
* Correctness -- self-checking implementation, good test coverage, etc. Just
  like the original implementation.
* Clarity; commented code, clear structure (hopefully doing a better job than
  the original implementation).
* Coming close-ish to the original implementation; clarifying the translation of
  typical C++ constructs to Rust.
