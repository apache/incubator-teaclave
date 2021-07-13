---
permalink: /docs/my-first-function
---

# My First Function

This documentation will guide you through executing your first function on the
Teaclave platform.

## Prerequisites

To run Teaclave, a hardware with Intel SGX support is needed. You can
check with this list of [supported hardware](https://github.com/ayeks/SGX-hardware).
Note that you sometimes need to configure BIOS to enable SGX. Additionally, you
need to install driver and platform software to run SGX applications. If you are
using Azure confidential computing VM, please refer to [this document](/docs/azure-confidential-computing/).
Otherwise, let install SGX driver first.

```
$ wget https://download.01.org/intel-sgx/sgx-linux/2.11/distro/ubuntu18.04-server/sgx_linux_x64_driver_2.6.0_b0a445b.bin
$ sudo ./sgx_linux_x64_driver_2.6.0_b0a445b.bin
$ ls /dev/isgx    # Make sure you have the SGX device
```

Then, install SGX architectural enclaves and quoting libraries for attestation.

```
$ sudo apt-get install libssl-dev libcurl4-openssl-dev libprotobuf-dev
$ echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
$ wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
$ sudo apt-get update && \
   sudo apt-get install libsgx-launch libsgx-urts libsgx-epid libsgx-urts libsgx-quote-ex  libsgx-aesm-quote-ex-plugin libsgx-aesm-epid-plugin
```

For more details, you can learn from
[Intel SGX Installation Guide](https://download.01.org/intel-sgx/sgx-linux/2.9/docs/Intel_SGX_Installation_Guide_Linux_2.9_Open_Source.pdf).

Docker and Docker Compose are also needed for building and trying Teaclave.

```
$ curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
$ sudo add-apt-repository \
   "deb [arch=amd64] https://download.docker.com/linux/ubuntu \
   $(lsb_release -cs) \
   stable"
$ sudo apt-get update && sudo apt-get install docker-ce docker-ce-cli containerd.io
$ sudo usermod -aG docker your-user-name
$ sudo curl -L "https://github.com/docker/compose/releases/download/1.27.4/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
$ sudo chmod +x /usr/local/bin/docker-compose
```

If you don't have an SGX supported hardware at hand, Teaclave can also run in
simulation mode. However some functions like remote attestation will be disabled
in this mode. Please start from [here](#simulation-mode) if you plan to try in
simulation mode.

## Clone and Build Teaclave

Clone the Teaclave repository:

```
$ git clone https://github.com/apache/incubator-teaclave.git
```

Since the building dependencies are a bit complicated, we suggest to build the
Teaclave platform with our docker images. You can learn more details about the
building environment from `Dockerfile` under the [`docker`](../docker)
directory.

Build the Teaclave platform using docker:

```
$ cd incubator-teaclave
$ docker run --rm -v $(pwd):/teaclave -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.14:latest \
   bash -c ". /root/.cargo/env && \
     . /opt/sgxsdk/environment && \
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
(i.e., Intel Attestation Service). At last, the AESM service needs to be restarted by
`sudo systemctl restart aesmd`.

```
$ sudo sed -i '/^#default quoting type = epid_linkable/s/^#//' /etc/aesmd.conf
$ sudo service aesmd restart
```

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
$ (cd docker && docker-compose -f docker-compose-ubuntu-1804-isgx.yml up --build)
Starting teaclave-authentication-service ... done
Starting teaclave-access-control-service ... done
Starting teaclave-scheduler-service      ... done
Starting teaclave-management-service     ... done
Starting teaclave-execution-service      ... done
Starting teaclave-frontend-service       ... done
Attaching to ...
```

## Invoke Function

We provide several examples to demonstrate the platform. Let's get started
with invoking a built-in function: echo, which is a simple function that takes one
input message and returns it.

This example is written in Python, and some dependencies are needed for the
remote attestation. They can be installed with `pip`:

```
$ pip3 install pyopenssl toml cryptography
```

### Built-in function

Then, run the echo example:

```
$ cd examples/python
$ PYTHONPATH=../../sdk/python python3 builtin_echo.py 'Hello, Teaclave!'
[+] registering user
[+] login
[+] registering function
[+] creating task
[+] approving task
[+] invoking task
[+] getting result
[+] done
[+] function return:  b'Hello, Teaclave!'
```

If you see above log, this means that the function is successfully invoked in Teaclave.

### Define my own function

The previous example is to demonstrate invoking the built-in echo function. In
Teaclave, you can also register and invoke a function written by yourself.
For example, we can implement an echo function in Python like this:

```
$ cat mesapy_echo_payload.py
def entrypoint(argv):
    assert argv[0] == 'message'
    assert argv[1] is not None
    return argv[1]
```

Then run the mesapy echo example:
```
$ PYTHONPATH=../../sdk/python python3 mesapy_echo.py mesapy_echo_payload.py 'Hello, Teaclave!'
[+] registering user
[+] login
[+] registering function
[+] creating task
[+] approving task
[+] invoking task
[+] getting result
[+] done
[+] function return:  b'Hello, Teaclave!'
```

## Simulation Mode

Clone and build Teaclave (with the `-DSGX_SIM_MODE=ON` option in `cmake`).
Note that if you are using Docker for Mac,
[increase the size of allocated memory](https://docs.docker.com/docker-for-mac/) to
avoid compilation error caused by out-of-memory, e.g., reporting a "signal: 9,
SIGKILL: kill" error during the compilation.

```
$ git clone https://github.com/apache/incubator-teaclave.git
$ cd incubator-teaclave
$ docker run --rm -v $(pwd):/teaclave -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.14:latest \
   bash -c ". /root/.cargo/env && \
     . /opt/sgxsdk/environment && \
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
$ (cd docker && docker-compose -f docker-compose-ubuntu-1804-sgx-sim-mode.yml up --build)
```

Install dependencies for Python client.

```
$ pip3 install pyopenssl toml cryptography
```

In simulation mode, run examples with `SGX_MODE=SW` environment variable.

```
$ cd examples/python
$ SGX_MODE=SW PYTHONPATH=../../sdk/python python3 builtin_echo.py 'Hello, Teaclave!'
[+] registering user
[+] login
[+] registering function
[+] creating task
[+] approving task
[+] invoking task
[+] getting result
[+] done
[+] function return:  b'Hello, Teaclave!'
```
