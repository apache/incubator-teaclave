FROM ubuntu:18.04

ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/linux-2.6/ubuntu18.04-server"

ENV SGX_LINUX_X64_SDK            sgx_linux_x64_sdk_2.6.100.51363.bin
ENV LIBSGX_ENCLAVE_COMMON        libsgx-enclave-common_2.6.100.51363-bionic1_amd64.deb
ENV LIBSGX_ENCLAVE_COMMON_DEV    libsgx-enclave-common-dev_2.6.100.51363-bionic1_amd64.deb
ENV LIBSGX_ENCLAVE_COMMON_DBGSYM libsgx-enclave-common-dbgsym_2.6.100.51363-bionic1_amd64.ddeb

ENV SGX_LINUX_X64_SDK_URL            "$SGX_DOWNLOAD_URL_BASE/$SGX_LINUX_X64_SDK"
ENV LIBSGX_ENCLAVE_COMMON_URL        "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON"
ENV LIBSGX_ENCLAVE_COMMON_DEV_URL    "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON_DEV"
ENV LIBSGX_ENCLAVE_COMMON_DBGSYM_URL "$SGX_DOWNLOAD_URL_BASE/$LIBSGX_ENCLAVE_COMMON_DBGSYM"

ENV DEBIAN_FRONTEND=noninteractive

ENV RUST_TOOLCHAIN nightly-2019-08-01

# install SGX dependencies
RUN apt-get update && apt-get install -q -y \
    build-essential \
    ocaml \
    ocamlbuild \
    automake \
    autoconf \
    libtool \
    wget \
    python \
    libssl-dev \
    libcurl4-openssl-dev \
    protobuf-compiler \
    libprotobuf-dev \
    sudo \
    kmod \
    vim \
    curl \
    git-core \
    libprotobuf-c0-dev \
    libboost-thread-dev \
    libboost-system-dev \
    liblog4cpp5-dev \
    libjsoncpp-dev \
    alien \
    uuid-dev \
    libxml2-dev \
    cmake \
    pkg-config \
    expect \
    systemd-sysv \
    gdb

# install other dependencies
RUN apt-get update && apt-get install -q -y \
    pypy \
    pypy-dev

RUN mkdir ~/sgx                                                               && \
    mkdir /etc/init                                                           && \
    cd ~/sgx                                                                  && \
    wget -O $LIBSGX_ENCLAVE_COMMON        "$LIBSGX_ENCLAVE_COMMON_URL"        && \
    wget -O $LIBSGX_ENCLAVE_COMMON_DEV    "$LIBSGX_ENCLAVE_COMMON_DEV_URL"    && \
    wget -O $LIBSGX_ENCLAVE_COMMON_DBGSYM "$LIBSGX_ENCLAVE_COMMON_DBGSYM_URL" && \
    wget -O $SGX_LINUX_X64_SDK            "$SGX_LINUX_X64_SDK_URL"

RUN cd ~/sgx                                  && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON            && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON_DEV        && \
    dpkg -i $LIBSGX_ENCLAVE_COMMON_DBGSYM     && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    echo 'source /opt/sgxsdk/environment' >> ~/.bashrc

# install Rust

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env                                                      && \
    rustup default $RUST_TOOLCHAIN                                          && \
    rustup component add rust-src rls rust-analysis clippy rustfmt          && \
    rustup target install aarch64-unknown-linux-gnu                         && \
    cargo install xargo                                                     && \
    cargo install sccache                                                   && \
    rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

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
