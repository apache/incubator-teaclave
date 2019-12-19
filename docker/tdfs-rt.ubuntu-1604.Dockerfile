FROM ubuntu:16.04

ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/sgx-linux/2.7.1/distro/ubuntu16.04-server"
ENV LIBSGX_ENCLAVE_COMMON        libsgx-enclave-common_2.7.101.3-xenial1_amd64.deb
ENV LIBSGX_ENCLAVE_COMMON_URL    "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON"

RUN apt-get update && apt-get install -q -y \
    libcurl3 \
    libprotobuf9v5 \
    wget

RUN wget -O $LIBSGX_ENCLAVE_COMMON "$LIBSGX_ENCLAVE_COMMON_URL" && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON                              && \
    rm $LIBSGX_ENCLAVE_COMMON

ADD release/services/tdfs /mesatee/
ADD release/services/tdfs.enclave.signed.so /mesatee/
ADD release/services/enclave_info.toml /mesatee/
ADD release/services/auditors /mesatee/auditors

ENTRYPOINT ["/mesatee/tdfs"]
