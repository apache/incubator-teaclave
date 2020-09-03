---
permalink: /docs/build-system
---

# Build System

The Teaclave's build system utilizes CMake to coordinate compilation, linking,
signing, etc. for various components written in different languages (e.g., Rust, C,
Python) for different targets (e.g., Linux and SGX). In this document, we will
introduce our build system in details.

## Quick Start

1. Download and [install](https://cmake.org/install/) CMake, the minimum
   required version is 3.10.
2. Open a shell and create a build directory.
  ```
  $ mkdir build && cd build
  ```
3. Run the following command to configure with `TEST_MODE` on.
  ```
  $ cmake -DTEST_MODE=ON ..
  ```
4. Build the whole platform.
  ```
  $ make
  ```

When making changes, run:

- `make format`: Format current source code.
- `make run-tests`: Make sure all tests are passed.

You can find more detailed configurations and targets in the following sections.

## Variables and Options

There are a lot of variables and options you can configure to control the build
system.

To set a variable or option, you can pass `-DXXX=` to `cmake`. For example,
`cmake -DTEST_MODE=ON ..` to enable the `TEST_MODE` option.

### Variables

- `SGX_SDK`: Set (or get from env vars) the path of Intel SGX SDK. Defaults to
  `/opt/sgxsdk`.
- `RUSTFLAGS`: Set (or get from env vars) flags passed to rustc.
- `MESAPY_VERSION`: Set the commit hash to the upstream MesaPy version.
- `RUSTUP_TOOLCHAIN`: Set the Rust toolchain version.
- `CMAKE_BUILD_TYPE`: Set the build type. Defaults to debug.

### Options

- `COV`: Build with coverage information. Defaults to OFF.
- `OFFLINE`: Compile Rust code with `cargo --offline`. Defaults to ON.
- `TEST_MODE`: Build with mock data and disabling some functions for testing.
  Defaults to OFF.
- `SGX_SIM_MODE`: Build in SGX simulation mode. Defaults to OFF.
- `DCAP`: Use DCAP instead of IAS as the attestation service. Defaults to OFF.
- `GIT_SUBMODULE`: Sync submodules with the upstream repositories. Defaults to
  ON.
- `CLP`: Enable `cargo clippy` to lint Rust code during the compilation.
  Defaults to OFF.
- `DOC`: Generate document with `cargo doc` during the compilation. Defaults to OFF.
- `USE_PREBUILT_MESAPY`: Whether to use the prebuilt MesaPy for SGX library. If
  set to OFF, will build the library from the source code. Defaults to ON.

## Targets

The followings are supported targets you can call with `make`. For example, to build a specific
service like execution service, you can just run `make teaclave_execution_service`.

### App/Enclave

An SGX application has two parts: the app part and the enclave part. You can
compile them separately or together using with these targets:

- `sgxapp-teaclave_{service_name}`: Build the app part of a service.
- `sgxlib-teaclave_{service_name}`: Build the enclave part of a service.
- `teaclave_{service_name}`: Build (compile, link and sign, etc.) the app and
  enclave of a service.
- `sgxapp-teaclave_{test_name}`: Build the app part of a test driver.
- `sgxlib-teaclave_{test_name}`: Build the enclave part of a test driver.
- `teaclave_{test_name}`: Build (compile, link, and sign, etc.) the app and
  enclave of a test driver.

These targets are automatically generated from the
`cmake/tomls/Cargo.sgx_{}.toml` files. Basically, they are:

- `test_name` can be: `function_tests`, `unit_tests`, `integration_tests`, etc.
- `service_name` can be: `access_control_service`, `authentication_service`,
  `storage_service`, `execution_service`, `frontend_service`,
  `management_service`, `scheduler_service`, etc.

### Bin

- `teaclave_cli`: Build the Teclave command line tool.
- `teaclave_dcap_ref_as`: Build the reference implementation of DCAP's
  attestation service.
- `teaclave_sgx_tool`: Build the SGX tool.

Above targets are automatically generated from the
`cmake/tomls/Cargo.unix_app.toml` files.

### Linting

- `format`: Format all code.
- `clippy`: Run `cargo clippy` for linting. Same with `make CLP=1`.

### Doc

- `doc`: Run `cargo doc` to generate documents. Same with `make DOC=1`.

### Tests

- `run-tests`: Run all test cases.
- `run-integration-tests`: Run integration tests only.
- `run-funtional-tests`: Run functional tests only.
- `run-examples`: Run all examples.
- `cov`: Aggregate coverage results and generate report, needs to config cmake
  with `-DCOV=ON`.

### Misc
- `clean`: Cleanup all building intermediates.

## Codebase

You can find source code to learn more about our build system in the
`CMakeLists.txt` file and the `cmake` directories.
