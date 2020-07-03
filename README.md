# Teaclave: A Universal Secure Computing Platform

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Coverage Status](https://coveralls.io/repos/github/apache/incubator-teaclave/badge.svg?branch=master)](https://coveralls.io/github/apache/incubator-teaclave?branch=master)

Apache Teaclave (incubating) is an open source ***universal secure computing***
platform, making computation on privacy-sensitive data safe and simple.

## Highlights

- **Security**:
  Teaclave adopts multiple security technologies to enable secure computing, in
  particular, Teaclave uses Intel SGX to serve the most security-sensitive tasks
  with *hardware-based isolation*, *memory encryption* and *attestation*.
  Also, Teaclave is written in Rust to prevent *memory-safety* issues.
- **Functionality**:
  Teaclave is provided as a *function-as-a-service platform*. With many built-in
  functions, it supports tasks like machine learning, private set intersection,
  crypto computation, etc. In addition, developers can also deploy and execute
  Python scripts in Teaclave. More importantly, unlike traditional FaaS,
  Teaclave supports both general secure computing tasks and *flexible
  single- and multi-party secure computation*.
- **Usability**:
  Teaclave builds its components in containers, therefore, it supports
  deployment both locally and within cloud infrastructures. Teaclave also
  provides convenient endpoint APIs, client SDKs and command line tools.
- **Modularity**:
  Components in Teaclave are designed in modular, and some like remote
  attestation can be easily embedded in other projects. In addition, Teaclave
  SGX SDK can also be used separately to write standalone SGX enclaves for other
  purposes.

## Getting Started

### Try Teaclave

- [My First Function](docs/my-first-function.md)
- [Write Functions in Python](docs/functions-in-python.md)
- [How to Add Built-in Functions](docs/builtin-functions.md)

### Design

- [Threat Model](docs/threat-model.md)
- [Mutual Attestation: Why and How](docs/mutual-attestation.md)
- [Access Control](docs/access-control.md)
- [Build System](docs/build-system.md)

### Contribute to Teaclave

- [Rust Development Guideline](docs/rust-guideline.md)
- [Development Tips](docs/development-tips.md)

### Codebase

- [Attestation](attestation)
- [Binder](binder)
- [Built-in Functions](function)
- [Client SDK](sdk)
- [Command Line Tool](cli)
- [Common Libraries](common)
- [Configurations in Teaclave](config)
- [Crypto Primitives](crypto)
- [Data Center Attestation Service](dcap)
- [Dockerfile and Compose File](docker)
- [Examples](examples)
- [Executor Runtime](runtime)
- [File Agent](file_agent)
- [Function Executors](executor)
- [Keys and Certificates](keys)
- [RPC](rpc)
- [Teaclave Services](services)
- [Teaclave Worker](worker)
- [Test Harness and Test Cases](tests)
- [Third-Party Dependency Vendoring](third_party)
- [Tool](tool)
- [Types](types)

### API Docs

- [Teaclave SGX SDK](https://teaclave.apache.org/docs/sgx-sdk/)
- [Teaclave Client SDK (Python)](https://teaclave.apache.org/docs/client-sdk-python/)
- [Crates in Teaclave (Enclave)](https://teaclave.apache.org/docs/crates-enclave/)
- [Crates in Teaclave (App)](https://teaclave.apache.org/docs/crates-app/)

## Contributing

Teaclave is open source in [The Apache Way](https://www.apache.org/theapacheway/),
we aim to create a project that is maintained and owned by the community. All
kinds of contributions are welcome. Read this [document](CONTRIBUTING.md) to
learn more about how to contribute. Thanks to our [contributors](CONTRIBUTORS.md).

## Community

- Join us on our [mailing list](https://lists.apache.org/list.html?dev@teaclave.apache.org).
- Follow us at [@ApacheTeaclave](https://twitter.com/ApacheTeaclave).
- See [more](COMMUNITY.md).
