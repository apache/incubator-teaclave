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
at this mode. Please start from [here](#simulation-mode) if you plan to try in
simulation mode.

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

## Setup Attestation Service

For simplicity, we use Intel Attestation Service (IAS) in this tutorial. To get
started, you need to enroll in Intel SGX Attestation Service in
Intel's [attestation service portal](https://api.portal.trustedservices.intel.com/EPID-attestation)
by subscribing the attestation service for development (linkable is preferred).
Then, you can find "SPID" and "Primary key" in the subscription details for
later usage.

There is one more setup if you are using linkable attestation service subscription.
Edit the `/etc/aesmd.conf` file and uncomment
the `default quoting type = epid_linkable` line to enable linkable quotes for EPID-based attestation service
(i.e., Intel Attestation Service). At last, the AESM service need to be restarted by
`sudo systemctl restart aesmd`.

## Launch Teaclave Services

Teaclave contains multiple services. To ease the deployment, you can use
[docker-compose](https://docs.docker.com/compose/) to manage all services in a
containerized environment.

Setup environment variables:

```
$ export AS_SPID="00000000000000000000000000000000"  # SPID from IAS subscription
$ export AS_KEY="00000000000000000000000000000000"   # Primary key/Secondary key from IAS subscription
$ export AS_ALGO="sgx_epid"                          # Attestation algorithm, sgx_epid for IAS
$ export AS_URL="https://api.trustedservices.intel.com:443"    # IAS URL
```

Launch all services with `docker-compose`:

```
$ (cd docker && docker-compose -f docker-compose-ubuntu-1804.yml up --build)
Starting teaclave-authentication-service ... done
Starting teaclave-access-control-service ... done
Starting teaclave-scheduler-service      ... done
Starting teaclave-management-service     ... done
Starting teaclave-execution-service      ... done
Starting teaclave-frontend-service       ... done
Attaching to ...
```

## Simulation Mode

To try Teaclave in SGX simulation mode, please install Intel SGX SDK first with instructions in
[Intel SGX Installation Guide](https://download.01.org/intel-sgx/sgx-linux/2.9/docs/Intel_SGX_Installation_Guide_Linux_2.9_Open_Source.pdf).

Then clone and build Teaclave (with the `-DSGX_SIM_MODE=ON` option in `cmake`).

```
$ git clone https://github.com/apache/incubator-teaclave.git
$ cd incubator-teaclave
$ docker run --rm -v $(pwd):/teaclave -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.9:latest \
   bash -c ". /root/.cargo/env && \
     mkdir -p build && cd build && \
     cmake -DTEST_MODE=ON -DSGX_SIM_MODE=ON .. && \
     make"
```

Since the attestation is disabled in the simulation mode, related environment
variables can be set to any values.

```
$ export AS_SPID="00000000000000000000000000000000"
$ export AS_KEY="00000000000000000000000000000000"
$ export AS_ALGO="sgx_epid"
$ export AS_URL="https://api.trustedservices.intel.com:443"
```

At last, launch all services with `docker-compose`:

```
$ (cd docker && docker-compose -f docker-compose-ubuntu-1804.yml up --build)
```
