---
permalink: /docs/functions-in-python
---

# Write Functions in Python

The Teaclave platform provides a convenient way to register a customized
function written in Python, and the function is interpreted at runtime in an
isolated trusted execution environment (i.e., Intel SGX).

## Entrypoint

Here is an simple example of an echo function:

```python
def entrypoint(argv):
    assert argv[0] == 'message'
    assert argv[1] is not None
    return argv[1]
```

The `entrypoint` function defined above is the "entrypoint" to executing the
function. It takes one argument which is a list of arguments of this echo
function. The return value of the `entrypoint` function will be passed back to
the client.

::: tip NOTE
Note that the function arguments in key-value format passed from the platform
are flattened into a list. For example, the `{"message": "Hello, Teaclave!"}`
arguments will become `"message"` (`argv[0]`) and `"Hello, Teaclave!"`
(`argv[1]`).
:::

## Modules

Current Python executor (i.e., MesaPy) already supports many modules of the
original Python standard library such as `marshal`, `math`, `binascii`,
`itertools`, `micronumpy`. You can find a full list of available modules in the
[document of MesaPy for SGX](https://github.com/mesalock-linux/mesapy/blob/sgx/sgx/README.md).

Besides these modules for general computation, you may curious about doing file
I/O in customized Python function. We provides APIs to integrated with the
executor runtime to read/write files registered along with the task. You can
either open a file through the `teaclave_open` function or with the `teaclave`
module like this:

```python
# open input via built-in teaclave_open
with teaclave_open("input_file", "rb") as f:
    line = f.readline()

# open input via teaclave module
from teaclave import open
with open("output_file", "wb") as f:
    f.write("This message is from Mesapy!")
```

Either function will give an `file` object in Python, you can use it to read
lines or write data. And the first argument is the key of the registered
input/output files.

You can learn more about advanced usages in the example of
[logistic regression in Python](https://github.com/apache/incubator-teaclave/tree/master/examples/python).
