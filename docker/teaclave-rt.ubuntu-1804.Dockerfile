FROM ubuntu:18.04

ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/sgx-linux/2.7.1/distro/ubuntu18.04-server"
ENV LIBSGX_ENCLAVE_COMMON        libsgx-enclave-common_2.7.101.3-bionic1_amd64.deb
ENV LIBSGX_ENCLAVE_COMMON_URL    "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON"

RUN apt-get update && apt-get install -q -y \
    libcurl4-openssl-dev \
    libprotobuf-dev \
    curl \
    pkg-config

RUN echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | \
  tee /etc/apt/sources.list.d/intel-sgx.list
RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -

RUN apt-get update && apt-get install -q -y \
    libsgx-launch libsgx-urts libsgx-quote-ex
RUN mkdir /etc/init

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
