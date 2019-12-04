FROM ubuntu:16.04

ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/linux-2.6/ubuntu16.04-server"

ENV SGX_LINUX_X64_SDK            sgx_linux_x64_sdk_2.6.100.51363.bin
ENV LIBSGX_ENCLAVE_COMMON        libsgx-enclave-common_2.6.100.51363-xenial1_amd64.deb
ENV LIBSGX_ENCLAVE_COMMON_DEV    libsgx-enclave-common-dev_2.6.100.51363-xenial1_amd64.deb

ENV SGX_LINUX_X64_SDK_URL            "$SGX_DOWNLOAD_URL_BASE/$SGX_LINUX_X64_SDK"
ENV LIBSGX_ENCLAVE_COMMON_URL        "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON"
ENV LIBSGX_ENCLAVE_COMMON_DEV_URL    "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON_DEV"

ENV DEBIAN_FRONTEND=noninteractive

ENV RUST_TOOLCHAIN nightly-2019-08-01

# install SGX dependencies
RUN apt-get update && apt-get install -q -y \
    build-essential \
    ocaml \
    automake \
    autoconf \
    libtool \
    wget \
    python \
    libssl-dev \
    libcurl3 \
    libprotobuf9v5

RUN mkdir ~/sgx                                                               && \
    cd ~/sgx                                                                  && \
    wget -O $LIBSGX_ENCLAVE_COMMON        "$LIBSGX_ENCLAVE_COMMON_URL"        && \
    wget -O $LIBSGX_ENCLAVE_COMMON_DEV    "$LIBSGX_ENCLAVE_COMMON_DEV_URL"    && \
    wget -O $SGX_LINUX_X64_SDK            "$SGX_LINUX_X64_SDK_URL"

RUN cd ~/sgx                                  && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON            && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON_DEV        && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    echo 'source /opt/sgxsdk/environment' >> ~/.bashrc

RUN rm -rf ~/sgx

# install Rust and its dependencies

RUN apt-get update && apt-get install -q -y curl pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y   && \
    . $HOME/.cargo/env                                                        && \
    rustup default $RUST_TOOLCHAIN                                            && \
    rustup component add rust-src rls rust-analysis clippy rustfmt            && \
    echo 'source $HOME/.cargo/env' >> ~/.bashrc                               && \
    cargo install xargo                                                       && \
    rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# install other dependencies for building

RUN apt-get update && apt-get install -q -y \
    software-properties-common \
    apt-transport-https \
    ca-certificates \
    git \
    pypy \
    pypy-dev

RUN wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null | apt-key add - && \
    apt-add-repository 'deb https://apt.kitware.com/ubuntu/ xenial main' && \
    apt-get update && \
    apt-get install -q -y cmake

# install dependencies for testing and coverage

RUN apt-get update && apt-get install -q -y \
    lsof \
    procps \
    lcov \
    llvm \
    curl

# clean up apt caches

RUN apt-get clean && \
    rm -fr /var/lib/apt/lists/* /tmp/* /var/tmp/*
