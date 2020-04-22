# Teaclave Docker

This directory contains the docker infrastructure for build and runtime
environment. Note that you must mount SGX device and ASEM domain socket into the
container environment to use SGX feature.

## Build

The build dockerfile (`build.*.Dockerfile`) only contains minimal dependencies
to build and test the project. To use them, you can directly use pre-built
docker images from Docker Hub with:

```
$ docker run --rm \
  --device=/dev/isgx \
  -v/var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
  -v`pwd`:/teaclave \
  -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9:latest \
  /bin/bash
```

or you can also build the image by yourself with `docker build`:

```
$ docker build -t teaclave-build - < build.*.Dockerfile
```
and run:

```
$ docker run --rm \
  --device=/dev/isgx \
  -v/var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
  -v`pwd`:/teaclave \
  -w /teaclave \
  -it teaclave/teaclave-build \
  /bin/bash
```

## Runtime

Teaclave contains many services, we put services, config and related
resources into one docker image
(`teaclave-rt.ubuntu-1804.Dockerfile`). To make the deployment
simpler, we recommend to use [docker-compose](https://docs.docker.com/compose/)
to manage all services. Since the remote attestation is required for all
services, you should setup the attestation service configurations
before start the services. You can use env vars or set them in the
`docker-compose-ubuntu-1804.yml` file.

Here is an example to start all services.

```
$ export AS_SPID="00000000000000000000000000000000"
$ export AS_KEY="00000000000000000000000000000000"
$ export AS_ALGO="sgx_epid"
$ export AS_URL="https://api.trustedservices.intel.com:443"

$ docker-compose -f docker-compose-ubuntu-1804.yml up
Starting teaclave-authentication-service ... done
Starting teaclave-access-control-service ... done
Starting teaclave-scheduler-service      ... done
Starting teaclave-management-service     ... done
Starting teaclave-execution-service      ... done
Starting teaclave-frontend-service       ... done
Attaching to ...
```
