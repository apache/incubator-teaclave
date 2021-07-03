---
permalink: /docs/executing-wasm
---

# Executing WebAssembly in Teaclave

Teaclave integrates WebAssembly Micro Runtime as an executor, which can
interpret WebAssembly bytecode in a sandboxed environment. Theoretically, source
code written in any language which can be compiled to WebAssembly should also be
executable in Teaclave. However, in order to be more secure, Teaclave cannot
provide interfaces such as syscalls to legacy applications, so the source code
should:

1. Be self-contained: not depending on libraries which are not provided by
   Teaclave, including standard libraries.
2. Contains no syscall: no system call-related code.
3. Implement required interface: exporting an `int entrypoint(int argc, char*
   argv[])` function which is compatible with Teaclave WAMR calling convention
   (see examples for more details).

Currently, [Teaclave file system APIs](../sdk/payload/wasm) are supported. We
also provide examples for compiling and executing from source code of various
languages.

::: tip NOTE
In current Teaclave client SDK, when passing arguments to the registered
function, each `(key, value)` pair is converted into two string pointers in
`argv` and you should expect `argc` is as twice as the actual number of
arguments. The calling convention is subject to further changes.
:::


## From C

`clang` supporting `wasm32` can be used for compiling Teaclave-compatible WASM
bytecode. Remember to add following options while compiling:

```sh
--target=wasm32 \
-nostdlib \
-Wl,--no-entry \
-Wl,--export-all, \
-Wl,--allow-undefined 
```

You can also use `clang` provided by
[wasi-sdk](https://github.com/WebAssembly/wasi-sdk), and the option
`--target=wasm32` is not needed for this version. We also provide an [example
payload written in
C](https://github.com/apache/incubator-teaclave/tree/master/examples/python/wasm_c_millionaire_problem_payload).

## From Rust

First of all, your cargo should support `wasm32` target and `wasm-gc` is
required to reduce the size of generated binary. You can easily run the
following commands to install dependencies:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-gc
```

There should be an exported `entrypoint` function in the source code, so you can
simply use `cargo` to create a new library and generate it with `cargo build
--target wasm32-unknown-unknown`. Please also add `crate-type = ["cdylib"]` in
the `[lib]` section into your `Cargo.toml` file to let cargo generate WASM file.
To reduce the size of WASM file, run:

```sh
cargo build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/[WASM FILENAME]
```

For detailed optimization options and function signature, please refer to the
[example payload](https://github.com/apache/incubator-teaclave/tree/master/examples/python/wasm_rust_psi_payload).

## References

- [Compiling Rust to WebAssembly: A Simple Example](https://depth-first.com/articles/2020/06/29/compiling-rust-to-webassembly-a-simple-example/)