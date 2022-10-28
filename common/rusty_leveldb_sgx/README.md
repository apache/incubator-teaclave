# rusty-leveldb-sgx
[![crates.io](https://img.shields.io/crates/v/rusty-leveldb.svg)](https://crates.io/crates/rusty-leveldb)

A fully compatible implementation of LevelDB in Rust. (any incompatibility is a
bug!) Be able to run inside SGX.

The implementation is very close to the original; often, you can see the same
algorithm translated 1:1, and class (struct) and method names are similar or
the same.

**NOTE: I do not endorse using this library for any data that you care about.**
I do care, however, about bug reports.

## Status

Working well, with a few rare bugs (see leveldb-rs issues).

## Goals

Some of the goals of this implementation are

* As few copies of data as possible; most of the time, slices of bytes (`&[u8]`)
  are used. Owned memory is represented as `Vec<u8>` (and then possibly borrowed
  as slice). Zero-copy is not always possible, though, and sometimes simplicity is favored.
* Correctness -- self-checking implementation, good test coverage, etc. Just
  like the original implementation.
* Clarity; commented code, clear structure (hopefully doing a better job than
  the original implementation).
* Coming close-ish to the original implementation; clarifying the translation of
  typical C++ constructs to Rust, and doing a better job at helping understand the internals.
