# Rust Development Guideline

This doc defines some guidelines for developing the MesaTEE project in Rust.

## Style

We use `rustfmt` and `clippy` to format and lint all Rust code. Mostly, we use
the default configurations, but there are a couple of custom settings and lint
exceptions. Our CI will check the format/lint issues and deny all warnings by
default. Simply run `make format` to format all code before submitting a PR. If
you still have some doubts of the `clippy` error, feel free to point out and add
an exception.

## Unsafe Rust

Using unsafe Rust is extremely dangerous, and may break Rust's strong
memory-safety guarantees. Therefore, we want to keep unsafe Rust as minimal as
possible. Sometime (very rare) using unsafe Rust can significant improve
performance, the unsafe code should *well documented* and *explain the
rationales*. For contributors and reviewers, pay attention to the unsafe code
and carefully check whether the pre-conditions and post-conditions are still
hold.

## Error handling

We have designed a simple and easy error handling mechanism in MesaTEE Core. It
can help to easyly convert different error types to `mesatee_core::Error`, and
can also preserve error semantics.

Using `unwrap` to get a value from an optional type may introduce runtime panic.
Therefore, properly using the error handling mechanism provided by Rust can make
the system robust and clean. In some cases, optional value can never be `None`
internally, `unwrap` can be used with a comment explaining the assumptions and
reasons. The same rule also applies to `panic` and similar functions which may
cause runtime panic.

## Third-party crates

To ensure the quality, stability and compatibility of upstream crates, all
third-party crates (especially for ported SGX-compatible crates) used in MesaTEE
are vendored in the `third_party` directory. Please refer to the `crates-sgx`
repo and choose specific versions of vendored crates.
