---
permalink: /docs/codebase/dcap
---

# Data Center Attestation Service

This directory includes a reference implementation of data center attestation
service using
[Intel SGX Data Center Attestation Primitives](https://software.intel.com/en-us/blogs/2019/05/21/intel-sgx-datacenter-attestation-primitives) (DCAP),
which allows third-parties to create their own attestation infrastructure for
the datacenter and cloud. Compared to Intel Attestation Service (IAS), DCAP
Attestation Service is for environment where internet services is not accessible
and entities who are unwilling to outsource trust decisions to third-parties
(like Intel's IAS).

By default, Intel Attestation Service (IAS) will be used for attestation in
Teaclave. To use DCAP instead of IAS, you have to first build Teaclave with DCAP
enabled (by appending `-DDCAP=ON` option to `cmake`) and deploy in
infrastructure with DCAP supported.

The Intel's [DCAP Installation Guide](https://download.01.org/intel-sgx/sgx-dcap/1.14/linux/docs/Intel_SGX_SW_Installation_Guide_for_Linux.pdf)
contains instructions to install essential dependencies for developers. Also,
you need to prepare environment in your infrastructure before deploying a
DCAP-enabled application.
