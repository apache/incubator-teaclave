# Rust Development Guideline

This doc defines some guidelines for developing Teaclave in Rust.

## Style

We use `rustfmt` and `clippy` to format and lint all Rust code. Mostly, we use
the default configurations, but there are a couple of custom settings and lint
exceptions. The exceptions should be defined along with the code. Our CI will
check the format/lint issues and deny all warnings by default. Simply run `make
format` to format all code and `make CLP=1` to lint code before submitting a PR.
If you still have some doubts of the `clippy` error, feel free to point out and
add an exception.

## Elegant APIs

Elegantly designed functions and APIs will make the project readable and
user-friendly. Basically, we follow naming conventions and API design patterns
of Rust standard library. There's no official guideline, but here are several
articles or docs for reference:

  - [Rust API guidelines](https://rust-lang.github.io/api-guidelines/)
  - [Rust Design Patterns](https://github.com/rust-unofficial/patterns)
  - [Elegant Library APIs in Rust](https://deterministic.space/elegant-apis-in-rust.html#what-makes-an-api-elegant)

## Unsafe Rust

Using unsafe Rust is extremely dangerous, and may break Rust's strong
memory-safety guarantees. Therefore, we want to keep unsafe Rust as minimal as
possible. Sometime (very rare) using unsafe Rust can significant improve
performance, the unsafe code should *well documented* and *explain the
rationales*. For contributors and reviewers, pay attention to the unsafe code
and carefully check whether the pre-conditions and post-conditions are still
hold.

## Error Handling

Using `unwrap` or `expect` to get a value from an optional type may introduce
runtime panic. Therefore, properly using the error handling mechanism provided
by Rust can make the system robust and clean. In some cases, optional value can
never be `None` internally, `unwrap` can be used with a comment explaining the
assumptions and reasons. The same rule also applies to `panic` and similar
functions which may cause runtime panic.

## Third-Party Crates

To ensure the security, stability and compatibility of upstream crates, all
third-party crates (especially for ported SGX-compatible crates) used in
Teaclave are vendored in the `third_party` directory. Please refer to the
`crates-sgx` and `crates-io` repo and choose specific versions of vendored
crates.
