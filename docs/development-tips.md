---
permalink: /docs/development-tips
---

# Development Tips

## RLS/rust-analyzer and IDEs

The most common question on developing Teaclave is how to use Rust IDEs to
improve help the development, e.g., code completions, type hints and cross
references. Internally, these features are supported by either
[RLS](https://github.com/rust-lang/rls) or
[rust-analyzer](https://github.com/rust-analyzer/rust-analyzer). Unfortunately,
these features are not supported in Teaclave's codebase out-of-box.
The reason is that Teaclave has components targeting different environments (SGX
enclave and Linux app) which need different set of dependencies (SGX crates and
vanilla crates). To support this flexible building and linking process, we are
using cmake for our [build system](build-system.md). However, there are still
ways to workaround and let the analyzer understand the project structures.

When developing SGX enclaves and corresponding dependent crates, you need to
prepare a `Cargo.toml` in the root directory to help the analyzer. This
`Cargo.toml` file can be copied from our build system:
`cmake/tomls/Cargo.sgx_trusted_lib.toml`. Similarly, when developing the app
parts, you can copy the `cmake/tomls/Cargo.sgx_untrusted_lib.toml` file to the
root directory as `Cargo.toml`. For standalone Rust applications such as CLI, no
`Cargo.toml` is needed. After the preparation of `Cargo.toml` in root,
RLS/rust-analyzer can understand the projects finally. You will see type hints
and cross references using IDEs with extensions.
