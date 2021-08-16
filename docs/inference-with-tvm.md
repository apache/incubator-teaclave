---
permalink: /docs/inference-with-tvm
---

# Inference Task with TVM in Teaclave

Inference and model training are two important topics in machine learning.
Thanks to [TVM](https://tvm.apache.org/) and [WebAssembly
Executor](https://teaclave.apache.org/docs/executing-wasm/), Teaclave is now
able to run the former—inference tasks. TVM can convert a model (or computation
graph) to an intermediate representation (IR) defined by TVM, and compile the
binary of this model from the IR. Since TVM recruits LLVM to emit binary code
and LLVM support WebAssembly as backend, Teaclave's WebAssembly Executor can
then execute the model's binary with additional lightweight runtime provided by
TVM. 

Although TVM has already provided an [wasm-standalone example
app](https://github.com/apache/tvm/tree/main/apps/wasm-standalone), we still
cannot copy and run it in Teaclave due to lack of WASI support and specific
context file interface. This document mainly focuses on the *what's different*
in Teaclave and we will finally build a MNIST inference function for Teaclave.

## Preparing TVM and Dependencies

All the dependencies has been installed or built in our docker image. If you do
not want to waste time on this step, you can skip this section with [our
image](https://hub.docker.com/layers/teaclave/teaclave-build-ubuntu-1804-sgx-2.14/0.1.2/images/sha256-7573f25acecddca48c57b20acb6f0fe9fe505503c33a9dc9905470f95ebd7829)
prepared.

TVM provides detailed build instruction in [the
document](https://tvm.apache.org/docs/install/from_source.html). Besides the
dependencies listed on their website, we also need to install (e.g. on Ubuntu
18.04) these packages to build the example.

```sh
sudo apt install protobuf-compiler libprotoc-dev llvm-10 clang-10
pip3 install onnx==1.9.0 numpy decorator attrs spicy
```

::: tip NOTE 
At the time of writing this document, latest `onnx` cannot work
because it depends on a higher version `protobuf`, which is not provided by
Ubuntu 18.04. We tested TVM with commit hash
`df06c5848f59108a8e6e7dffb997b4b659b573a7`. Later versions may work, but commits
older than this one hardly work.
:::

## Compiling WASM Library

TVM offers a set of Python APIs for downloading, building, and testing the
model. Specifically, to compile a graph into binary, we need to:

1. Download the model
2. Determine the name and shape of input
3. Generate TVM IR module
4. Compile(build) to LLVM WebAssembly target
5. Save the object, graph, and param files
6. Archive the object(`llvm-ar`) to a static library

After completing these steps, we will generate a static library with the
`PackedFunc` exported for inference task.

The complete example build script can be found
[here](https://github.com/apache/incubator-teaclave/blob/master/examples/python/wasm_tvm_mnist_payload/build_lib.py).

## Bridging with Teaclave

Although the library is in WebAssembly, we can not use it directly in Teaclave
because it lacks parameters, and the interfaces is also not compatible with
Teaclave. So we need a wrapper program which contains a small runtime for the
compiled computation graph. This wrapper should:

- Load model parameters and graph json
- Link with the graph library generated in the previous section
- Export an entrypoint which is compatible with the Teaclave's interface
- Read input data(image) using Teaclave's API and convert it to tensor
- Call the graph function and get the result back

Our wrapper is dependent on TVM's Rust APIs. We use `GraphExecutor` to achieve
calling to the graph library. Detailed mechanisms are explained in [TVM's
example](https://github.com/apache/tvm/tree/main/apps/wasm-standalone). our
example can be found
[here](https://github.com/apache/incubator-teaclave/tree/master/examples/python/wasm_tvm_mnist_payload).


::: tip NOTE 
To compile a Teaclave-compatible WASM binary, please make sure your
Rust version > 1.53. We tested on 1.54 stable.
:::

## Execute the function

Just like any other Teaclave function, users need to prepare a simple Python
script to pass the function and data to Teaclave, and then get the result back.
The script of this example is
[here](https://github.com/apache/incubator-teaclave/blob/master/examples/python/wasm_tvm_mnist.py).

::: tip NOTE
To compile a Teaclave-compatible WASM binary, please make sure your
Rust version > 1.53. We tested on 1.54 stable.
:::
