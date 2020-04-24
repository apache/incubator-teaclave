FROM ubuntu:18.04

ENV VERSION 2.9.101.2-bionic1
ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/sgx-linux/2.9.1/distro/ubuntu18.04-server"
ENV SGX_LINUX_X64_SDK sgx_linux_x64_sdk_2.9.101.2.bin
ENV SGX_LINUX_X64_SDK_URL "$SGX_DOWNLOAD_URL_BASE/$SGX_LINUX_X64_SDK"

RUN apt-get update && apt-get install -q -y \
    libcurl4-openssl-dev \
    libprotobuf-dev \
    curl \
    pkg-config \
    wget

RUN echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | \
  tee /etc/apt/sources.list.d/intel-sgx.list
RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -

RUN apt-get update && apt-get install -q -y \
    libsgx-launch=$VERSION \
    libsgx-urts=$VERSION \
    libsgx-quote-ex=$VERSION
RUN mkdir /etc/init

# Install Intel SGX SDK for libsgx_urts_sim.so
RUN wget $SGX_LINUX_X64_SDK_URL               && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt/intel' | ./$SGX_LINUX_X64_SDK && \
    rm $SGX_LINUX_X64_SDK                     && \
    echo 'source /opt/sgxsdk/environment' >> /etc/environment
ENV LD_LIBRARY_PATH=/opt/intel/sgxsdk/sdk_libs

ADD release/services/teaclave_frontend_service /teaclave/
ADD release/services/teaclave_frontend_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_authentication_service /teaclave/
ADD release/services/teaclave_authentication_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_management_service /teaclave/
ADD release/services/teaclave_management_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_scheduler_service /teaclave/
ADD release/services/teaclave_scheduler_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_access_control_service /teaclave/
ADD release/services/teaclave_access_control_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_storage_service /teaclave/
ADD release/services/teaclave_storage_service_enclave.signed.so /teaclave/

ADD release/services/teaclave_execution_service /teaclave/
ADD release/services/teaclave_execution_service_enclave.signed.so /teaclave/

ADD release/services/enclave_info.toml /teaclave/
ADD release/services/auditors /teaclave/auditors
