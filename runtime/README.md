---
permalink: /runtime
---

# Executor Runtime

This directory contains implementations of executor's runtime. The executor
runtime provides interfaces (I/O) between executors (in trusted execution
environment) and external components (in untrusted world like file system). The
interfaces are defined in the `TeaclaveRuntime` traits. Currently, we have two
runtime implementations: `DefaultRuntime` and `RawIoRuntime`. By default,
Teaclave provides a runtime called `DefaultRuntime`, which bridges interfaces to
our secure file system implementation (i.e., *protected file*). While
`RawIoRuntime` is only for debugging, which does not encrypt any I/O.
