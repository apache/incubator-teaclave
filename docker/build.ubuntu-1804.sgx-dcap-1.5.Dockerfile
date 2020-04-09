FROM ubuntu:18.04

ENV VERSION 1.5.100.2-bionic1
ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/sgx-dcap/1.5/linux/distro/ubuntuServer18.04/"
ENV SGX_LINUX_X64_SDK sgx_linux_x64_sdk_2.9.100.2.bin
ENV SGX_LINUX_X64_SDK_URL "$SGX_DOWNLOAD_URL_BASE/$SGX_LINUX_X64_SDK"

ENV DEBIAN_FRONTEND=noninteractive

ENV RUST_TOOLCHAIN nightly-2020-03-12

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
    python3 \
    libssl-dev \
    libcurl4-openssl-dev \
    libprotobuf-dev \
    curl \
    pkg-config

RUN echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu bionic main' | \
    tee /etc/apt/sources.list.d/intel-sgx.list
RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -
RUN apt-get update && apt-get install -y \
    libsgx-dcap-ql=$VERSION \
    libsgx-dcap-default-qpl=$VERSION \
    libsgx-dcap-ql-dbgsym=$VERSION \
    libsgx-dcap-default-qpl-dbgsym=$VERSION \
    libsgx-urts=2.9.100.2-bionic1 \
    libsgx-enclave-common=2.9.100.2-bionic1 \
    libsgx-enclave-common-dev=2.9.100.2-bionic1 \
    libsgx-enclave-common-dbgsym=2.9.100.2-bionic1 \
    libsgx-quote-ex=2.9.100.2-bionic1 \
    libsgx-quote-ex-dev=2.9.100.2-bionic1 \
    libsgx-dcap-ql-dev=$VERSION \
    libsgx-dcap-default-qpl-dev=$VERSION
RUN mkdir /var/run/aesmd && mkdir /etc/init
RUN wget $SGX_LINUX_X64_SDK_URL               && \
    chmod u+x $SGX_LINUX_X64_SDK              && \
    echo -e 'no\n/opt' | ./$SGX_LINUX_X64_SDK && \
    rm $SGX_LINUX_X64_SDK                     && \
    echo 'source /opt/sgxsdk/environment' >> ~/.bashrc

# install Rust and its dependencies

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y   && \
    . $HOME/.cargo/env                                                        && \
    rustup default $RUST_TOOLCHAIN                                            && \
    rustup component add rust-src rls rust-analysis clippy rustfmt            && \
    echo 'source $HOME/.cargo/env' >> ~/.bashrc                               && \
    rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# install other dependencies for building

RUN apt-get update && apt-get install -q -y \
    git \
    cmake \
    pypy \
    pypy-dev

# install dependencies for testing and coverage

RUN apt-get update && apt-get install -q -y \
    lsof \
    procps \
    lcov \
    llvm \
    curl \
    iproute2 \
    python3-pip

RUN pip3 install pyopenssl toml cryptography

# clean up apt caches

RUN apt-get clean && \
    rm -fr /var/lib/apt/lists/* /tmp/* /var/tmp/*
