# MesaTEE Docker

This directory contains the docker infrastructure for build and runtime
environment. Both Ubuntu 16.04 and 18.04 images are provided. Note that
you must mount SGX device and ASEM domain socket into the container
environment to use SGX feature.

## Build

The build dockerfile (`build.ubuntu-{1604,1804}.Dockerfile`) only contains
minimal dependencies to build and test the project. To use them, you can
directly use pre-built docker images from Docker Hub with:

```
$ docker run --rm \
  --device=/dev/isgx \
  -v/var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
  -v`pwd`:/mesatee \
  -w /mesatee \
  -it mesalocklinux/mesatee-build-ubuntu-1804
```

or you can also build the image by yourself with `docker build`:

```
$ docker build -t mesatee-build-ubuntu-1804 - < build.ubuntu-1804.Dockerfile
```
and run:

```
$ docker run --rm \
  --device=/dev/isgx \
  -v/var/run/aesmd/aesm.socket:/var/run/aesmd/aesm.socket \
  -v`pwd`:/mesatee \
  -w /mesatee \
  -it mesatee-build-ubuntu-1804
```

## Runtime

MesaTEE contains many services, we have put each service, config and related
resources into different docker image
(`{tms,tdfs,kms,fns}-rt.ubuntu-{1604,1804}.Dockerfile`). To make the deployment
simpler, we recommend to use [docker-compose](https://docs.docker.com/compose/)
to manage all services. Since the remote attestation is required for all
services, you should setup the Intel Attestation Service ID (SPID) and key
before start the services. You can use env vars or set them in the
`docker-compose-ubuntu-{1604,1804}.yml` file.

```
$ export IAS_SPID=xxxxxx
$ export IAS_KEY=xxxxxx
$ cd docker && docker-compose -f docker-compose-ubuntu-1804.yml up
Starting docker_mesatee-tms_1  ... done
Starting docker_mesatee-tdfs_1 ... done
Starting docker_mesatee-kms_1  ... done
Starting docker_mesatee-fns_1  ... done
Attaching to docker_mesatee-kms_1, docker_mesatee-tms_1, docker_mesatee-tdfs_1, docker_mesatee-fns_1
```
