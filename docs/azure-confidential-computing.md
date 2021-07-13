---
permalink: /docs/azure-confidential-computing
---

# Deploying Teaclave on Azure Confidential Computing Virtual Machines

If you want to try Teaclave on an Intel-SGX enabled machine instead of in simulation mode,
Azure, as a cloud service provider, has provided [Intel-SGX enabled virtual machines](https://azure.microsoft.com/en-us/blog/dcsv2series-vm-now-generally-available-from-azure-confidential-computing/).
This tutorial will guide you to deploy Teaclave on Azure confidential computing VMs.

To get started, you need to create an Azure confidential computing VM. Please
refer to this documents: [Quickstart: Deploy an Azure confidential computing VM in the Azure portal](https://docs.microsoft.com/en-us/azure/confidential-computing/quick-create-portal).

Normally, the SGX driver will be pre-installed after successfully creating an
Azure confidential computing VM. Please use this command to check whether the
SGX driver (the `intel_sgx` kernel module) is properly installed.

```
$ ls /dev/sgx
enclave  provision
```

Then, install SGX architectural enclaves and quoting libraries for attestation.

```
$ sudo apt-get install libssl-dev libcurl4-openssl-dev libprotobuf-dev
$ echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
$ wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
$ sudo apt-get update && \
   sudo apt-get install libsgx-launch libsgx-urts libsgx-epid libsgx-urts libsgx-quote-ex  libsgx-aesm-quote-ex-plugin libsgx-aesm-epid-plugin
$ sudo sed -i '/^#default quoting type = epid_linkable/s/^#//' /etc/aesmd.conf
$ sudo service aesmd restart
```

Install Docker and Docker Compose.

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

Build Teaclave.

```
$ git clone https://github.com/apache/incubator-teaclave.git
$ cd incubator-teaclave
$ docker run --rm -v $(pwd):/teaclave -w /teaclave \
  -it teaclave/teaclave-build-ubuntu-1804-sgx-2.14:latest \
   bash -c ". /root/.cargo/env && \
     . /opt/sgxsdk/environment && \
     mkdir -p build && cd build && \
     cmake -DTEST_MODE=ON .. && \
     make"

```

Setup environments for remote attestation. We are using Intel's Attestation
Service and linkable quote, and you can request access from the
[Development (DEV) attestation service portal](https://api.portal.trustedservices.intel.com/EPID-attestation)
for testing.

```
export AS_ALGO=sgx_epid
export AS_KEY=XXX
export AS_SPID=XXX
export AS_URL=https://api.trustedservices.intel.com:443
```

Start Teaclave services.

```
(cd docker && docker-compose -f docker-compose-ubuntu-1804-intel-sgx.yml up --build --detach)
```

At last, try the hello world example.

```
$ sudo apt install python3-pip
$ pip3 install pyopenssl toml cryptography
$ cd examples/python
$ PYTHONPATH=../../sdk/python python3 builtin_echo.py 'Hello, Teaclave!'
[+] registering user
[+] login
[+] registering function
[+] creating task
[+] invoking task
[+] getting result
[+] done
[+] function return:  b'Hello, Teaclave!'
```

You can also open the port numbers of Teaclave's frontend/authentication
services in the Azure portal and run examples in another client machine with the
address this VM. Note that the client SDK needs enclave info (i.e., the
`enclave_info.toml` file) and attestation service's cert (i.e., the
`ias_root_ca_cert.pem` file) for attesting remote SGX services. The paths of
these files can be set in `examples/python/utils.py`.
