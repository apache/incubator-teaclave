---
permalink: /docs/codebase/examples
---

# Examples

In this directory, we implement some examples to illustrate how to register
input/output data for a function, create and invoke a task and get execution
results with the Teclave's client SDK in both single and multi-party setups.

Before trying these examples, please make sure all services in the Teaclave
platform has been properly launched. Also, for examples implemented in Python,
don't forget to generate protocol stub files and set the `PYTHONPATH` to the
`sdk` path so that the scripts can successfully import the `teaclave` module.

Generate stub files by grpcio-tools and grpclib.

```
python3 -m grpc_tools.protoc --proto_path=../../services/proto/src/proto --python_out=. --grpclib_python_out=. ../../services/proto/src/proto/{teaclave_authentication_service.proto,teaclave_frontend_service.proto,teaclave_common.proto}
```

For instance, use the following command to invoke an echo function in Teaclave:

```
$ PYTHONPATH=../../sdk/python python3 builtin_echo.py 'Hello, Teaclave!'
```

Please checkout the sources of these examples to learn more about the process of
invoking a function in Teaclave.

## Configuring URLs of Input/Output Files

In some of the examples, you will see URLs of input and output files pointing to
the `localhost` addresses. In real world, these URLs are addresses from file
system service providers (i.e., AWS S3). If you are using the Docker compose
file to start Teaclave services, a simple file system service are also included.
To use it, just change the URLs in the examples to
`http://teaclave-file-service:6789/path/to/the/file`.

Normally, the domain name is `teaclave-file-service`, and it can be found via
the `docker ps` command under the "NAMES" column:

```
CONTAINER ID || IMAGE    ||   COMMAND               || CREATED     || STATUS    || NAMES
XXXXXXXX     || python:3 || "./scripts/simple_ht…"  || 1 days ago  || Up 1 days || teaclave-file-service
```

Note that in a real-world case, URLs of input and output files should be
provided by the end-user. In the examples, we just embed these files for
demonstration and testing.
