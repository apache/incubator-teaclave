# My First Function

This documentation will guide you through executing your first function on the
Teaclave platform.

## Prerequisites

To run Teacalve, a hardware with Intel SGX support is needed. You can
check with this list of [supported hardware](https://github.com/ayeks/SGX-hardware).
Note that you need to configure BIOS to enable SGX sometime. Additionally, you
need to install driver and platform software to run SGX applications. Details
can found in
[Intel SGX Installation Guide](https://download.01.org/intel-sgx/sgx-linux/2.9/docs/Intel_SGX_Installation_Guide_Linux_2.9_Open_Source.pdf).

If you don't have an SGX supported hardware at hand, Teaclave can also run in
simulation mode. However some functions like remote attestation will be disable
at this mode.

## Clone and Build Teaclave

Clone the Teaclave repository:

```
$ git clone https://github.com/apache/incubator-teaclave.git
```

Since the building dependencies is a bit complicated, we suggest to build the
Teaclave platform with our docker images. You can learn more details about the
building environment from `Dockerfile` under the [`docker`](../docker)
directory.

Build the Teaclave platform using docker:

```
$ cd incubator-teaclave
$ docker run --rm -v $(pwd):/teaclave -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9:latest \
   bash -c ". /root/.cargo/env && \
     mkdir -p build && cd build && \
     cmake -DTEST_MODE=ON .. && \
     make"
```

To build in simulation mode, you can add `-DSGX_SIM_MODE=ON` to `cmake`.

### Launch Teaclave

Teaclave contains multiple services. To ease the deployment, you can use
[docker-compose](https://docs.docker.com/compose/) to manage all services in a
containerized environment.

Launch all services with `docker-compose`:

```
$ export AS_SPID="00000000000000000000000000000000"
$ export AS_KEY="00000000000000000000000000000000"
$ export AS_ALGO="sgx_epid"
$ export AS_URL="https://api.trustedservices.intel.com:443"

$ (cd docker && docker-compose -f docker-compose-ubuntu-1804.yml up --build)
Starting teaclave-authentication-service ... done
Starting teaclave-access-control-service ... done
Starting teaclave-scheduler-service      ... done
Starting teaclave-management-service     ... done
Starting teaclave-execution-service      ... done
Starting teaclave-frontend-service       ... done
Attaching to ...
```
