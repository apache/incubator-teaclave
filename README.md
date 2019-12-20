# Teaclave: A Universal Secure Computing Platform

Apache Teaclave (incubating) is an open source ***universal secure computing***
platform.

***Security***:
Teaclave adopts multiple security technologies to enable secure computing, in
particular, Teaclave uses Intel SGX to serve the most security-sensitive tasks
with *hardware-based isolation*, *memory encryption* and *attestation*.
Also, Teaclave is built in the Rust programming language to prevent
*memory-safety* issues.

***Functionality***:
Teaclave is provided as a *function-as-a-service platform* for secure computing.
With many useful built-in functions, it supports tasks such as machine learning,
private set intersection (PSI), crypto computation, etc. Developers can easily
deploy a Python script in the Teaclave's trusted execution environment. More
importantly, unlike traditional FaaS, Teaclave supports both general secure
computing tasks and *flexible multi-party secure computation*.

***Usability***:
Teaclave builds its components in containers, therefore, it supports deployment
both locally and within cloud infrastructures. Teaclave also provides client
SDKs and a command line tool.

Teaclave is originated from Baidu X-Lab (formerly named MesaTEE).

## Quick Start


Download and build Teaclave services, examples, SDK, and command line tool.

```
git clone https://github.com/apache/incubator-teaclave.git
docker run --rm -v$(pwd)/incubator-teaclave:/teaclave -w /teaclave -it teaclave/teaclave-build-ubuntu-1804:latest
mkdir -p build && cd build
cmake -DTEST_MODE=ON .. && make
```

Start all Teaclave services with
[Docker Compose](https://docs.docker.com/compose/) and detach into background.
Make
sure [SGX driver and PSW package](https://01.org/intel-software-guard-extensions/downloads)
are properly installed and you have got the
[SPID and key](https://api.portal.trustedservices.intel.com/EPID-attestation)
to connect Intel Attestation Service.

```
export IAS_SPID=xxx
export IAS_KEY=xxx
(cd docker && docker-compose -f docker-compose-ubuntu-1804.yml up --build --detach)
```

Try the "quickstart" example.

```
./release/examples/quickstart echo -e release/examples/enclave_info.toml -m "Hello, World!"
```

Shutdown all Teaclave services.

```
(cd docker && docker-compose -f docker-compose-ubuntu-1804.yml down)
```

## Contributing

Teaclave is open source in [The Apache Way](https://www.apache.org/theapacheway/),
we aim to create a project that is maintained and owned by the community. All
kinds of contributions are welcome.


## Community

Please subscribe our mailing list
[dev@teaclave.apache.org](https://lists.apache.org/list.html?dev@teaclave.apache.org)
for development related activities. To subscribe, send an email to
`dev-subscribe@teaclave.apache.org`.
