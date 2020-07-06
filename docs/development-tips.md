---
permalink: /docs/development-tips
---

# Development Tips

## RLS/rust-analyzer and IDEs

The most common question on developing Teaclave is how to use Rust IDEs to
improve the development experience, e.g., code completions, type hints and cross
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

## Logging

Teaclave utilizes the [`env_logger`](https://github.com/sebasmagri/env_logger/)
crate to configure the display of *debug logs* via environment variables.

Logging is controlled via the `TEACLAVE_LOG` environment variables and the value
of this variable is a comma-separated list of logging directives in the
`parth::to::module=level` form. For example, you can set the environment
`TEACLAVE_LOG=attestation=debug` before launching a service to print the debug
level (and higher-level) logs in the `attestation` module to stdout/stderr.
There are five logging levels: `error`, `warn`, `info`, `debug` and `trace`
where error represents the highest-priority log level. Furthermore, you can also
filter the results with regular expression by simply put `/` followed by a regex
in the directives in the environment variable. You can find more filter usages
in the `env_logger`'s
[document](https://docs.rs/env_logger/0.7.1/env_logger/index.html#filtering-results).


::: tip NOTE
To prevent sensitive information leakage through logging, for the release build,
we disable all logging (at build time) lower than the `info` level. That is,
only `error`, `warn` and `info` logs will be printed.
:::
