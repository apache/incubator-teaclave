---
permalink: /docs/codebase/worker
---

# Teaclave Worker

The worker layer in Teaclave is a thin layer to manage executors and runtimes.
There are several executors and runtime for different usage scenarios.
Developers can customize and register different executors in a worker.

This diagram demonstrates the relationship between the execution service,
worker, executor and runtime.

```
    +-----------------------------------+
    |        Execution Service          |
    |  +-----------------------------+  |
    |  |            Worker           |  |
    |  |  +----------+  +---------+  |  |
    |  |  | Executor |  | Runtime |  |  |
    |  |  +----------+  +---------+  |  |
    |  +-----------------------------+  |
    +-----------------------------------+
```

The execution service is a service instance to maintain communication with other
services through attested RPC, prepare data and related information for function
execution, execute a function with a *worker* and report execution result. The
worker will prepare a proper *executor* and *runtime* combination, and then
dispatch the function to the executor, which will eventually run the function.
At the same time, the runtime will help to manage input and output data of
functions and provide interfaces in executor.

Currently, there are several executors (e.g., mesapy, builtin) and runtime
(e.g., default, raw-io) are implemented and registered in worker. Please refer
to the docs of executor and runtime for more details.
