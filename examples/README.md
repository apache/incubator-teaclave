---
permalink: /docs/codebase/examples
---

# Examples

In this directory, we implement some examples to illustrate how to register
input/output data for a function, create and invoke a task and get execution
results with the Teclave's client SDK in both single and multi-party setups.

Before trying these examples, please make sure all services in the Teaclave
platform has been properly launched. Also, for examples implemented in Python,
don't forget to set the `PYTHONPATH` to the `sdk` path so that the scripts can
successfully import the `teaclave` module.

For instance, use the following command to invoke an echo function in Teaclave:

```
$ PYTHONPATH=../../sdk/python python3 builtin_echo.py 'Hello, Teaclave!'
```

Please checkout the sources of these examples to learn more about the process of
invoking a function in Teaclave.
