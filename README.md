# Teaclave: A Universal Secure Computing Platform

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/tag/apache/incubator-teaclave?label=release&sort=semver)](https://github.com/apache/incubator-teaclave/releases)
[![Coverage Status](https://coveralls.io/repos/github/apache/incubator-teaclave/badge.svg?branch=master)](https://coveralls.io/github/apache/incubator-teaclave?branch=master)
[![Homepage](https://img.shields.io/badge/site-homepage-blue)](https://teaclave.apache.org/)

**Note**: The `master` branch has been renamed to `legacy` and is deprecated. 
The Teaclave FaaS framework is no longer actively maintained as the community's focus has shifted to SDK development. 
The legacy codebase remains available for historical reference and to recognize early contributions.

Apache Teaclave (incubating) is an open source ***universal secure computing***
platform, making computation on privacy-sensitive data safe and simple.

## Highlights

- **Secure and Attestable**:
  Teaclave adopts multiple security technologies to enable secure computing. In
  particular, Teaclave uses Intel SGX to serve the most security-sensitive tasks
  with *hardware-based isolation*, *memory encryption* and *attestation*.
  Also, Teaclave is written in Rust to prevent *memory-safety* issues.
- **Function-as-a-Service**:
  Teaclave is provided as a *function-as-a-service platform*. With many built-in
  functions, it supports tasks like machine learning, private set intersection,
  crypto computation, etc. In addition, developers can also deploy and execute
  Python scripts in Teaclave. More importantly, unlike traditional FaaS,
  Teaclave supports both general secure computing tasks and *flexible
  single- and multi-party secure computation*.
- **Ease of Use**:
  Teaclave builds its components in containers, therefore, it supports
  deployment both locally and within cloud infrastructures. Teaclave also
  provides convenient endpoint APIs, client SDKs and command line tools.
- **Flexible**:
  Components in Teaclave are designed in a modular manner, and features like remote
  attestation can be easily embedded in other projects. In addition, Teaclave
  SGX SDK and Teaclave TrustZone SDK can also be used separately to write TEE
  apps for other purposes.

## Getting Started

### Try Teaclave

- [My First Function](docs/my-first-function.md)
- [Write Functions in Python](docs/functions-in-python.md)
- [How to Add Built-in Functions](docs/builtin-functions.md)
- [Deploying Teaclave on Azure Confidential Computing VM](docs/azure-confidential-computing.md)
- [Executing WebAssembly in Teaclave](docs/executing-wasm.md)
- [Running LibOS in Teaclave](docs/executing-in-occlum.md)

### Design

- [Threat Model](docs/threat-model.md)
- [Mutual Attestation: Why and How](docs/mutual-attestation.md)
- [Access Control](docs/access-control.md)
- [Build System](docs/build-system.md)
- [Teaclave Service Internals](docs/service-internals.md)
- [Adding Executors](docs/adding-executors.md)
- [Papers, Talks, and Related Articles](docs/papers-talks.md)

### Contribute to Teaclave

- [Release Guide](docs/release-guide.md)
- [Rust Development Guideline](docs/rust-guideline.md)
- [Development Tips](docs/development-tips.md)

## Teaclave Projects

This is the main repository for the Teaclave FaaS platform. There are several
sub-projects under Teaclave:

- [Teaclave SGX SDK](https://github.com/apache/incubator-teaclave-sgx-sdk)
- [Teaclave TrustZone SDK](https://github.com/apache/incubator-teaclave-trustzone-sdk)
- [Teaclave Java TEE SDK](https://github.com/apache/incubator-teaclave-java-tee-sdk)

## Contributing

Teaclave is open source in [The Apache Way](https://www.apache.org/theapacheway/),
we aim to create a project that is maintained and owned by the community. All
kinds of contributions are welcome. Read this [document](CONTRIBUTING.md) to
learn more about how to contribute. Thanks to our
[contributors](https://teaclave.apache.org/contributors/).

## Community

- Join us on our [mailing list](https://lists.apache.org/list.html?dev@teaclave.apache.org).
- Follow us at [@ApacheTeaclave](https://twitter.com/ApacheTeaclave).
- See [more](https://teaclave.apache.org/community/).
