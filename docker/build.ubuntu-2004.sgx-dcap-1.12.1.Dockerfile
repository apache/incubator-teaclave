# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

FROM ubuntu:20.04

ENV DCAP_VERSION 1.12.101.1-focal1
ENV VERSION 2.15.101.1-focal1
ENV SGX_DOWNLOAD_URL_BASE "https://download.01.org/intel-sgx/sgx-dcap/1.12.1/linux/distro/ubuntu20.04-server"
ENV SGX_LINUX_X64_SDK sgx_linux_x64_sdk_2.15.101.1.bin
ENV SGX_LINUX_X64_SDK_URL "$SGX_DOWNLOAD_URL_BASE/$SGX_LINUX_X64_SDK"

ENV DEBIAN_FRONTEND=noninteractive

ENV RUST_TOOLCHAIN nightly-2020-10-25

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

RUN echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | \
    tee /etc/apt/sources.list.d/intel-sgx.list
RUN curl -fsSL  https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | apt-key add -
RUN apt-get update && apt-get install -y -f \
    libsgx-dcap-ql=$DCAP_VERSION \
    libsgx-dcap-default-qpl=$DCAP_VERSION \
    libsgx-dcap-ql-dbgsym=$DCAP_VERSION \
    libsgx-dcap-default-qpl-dbgsym=$DCAP_VERSION \
    libsgx-dcap-quote-verify=$DCAP_VERSION \
    libsgx-dcap-quote-verify-dev=$DCAP_VERSION \
    libsgx-urts=$VERSION \
    libsgx-enclave-common=$VERSION \
    libsgx-enclave-common-dev=$VERSION \
    libsgx-enclave-common-dbgsym=$VERSION \
    libsgx-quote-ex=$VERSION \
    libsgx-quote-ex-dev=$VERSION \
    libsgx-dcap-ql-dev=$DCAP_VERSION \
    libsgx-dcap-default-qpl-dev=$DCAP_VERSION \
    libsgx-qe3-logic=$DCAP_VERSION \
    libsgx-pce-logic=$DCAP_VERSION \
    libsgx-uae-service=$VERSION
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
    rustup target add wasm32-unknown-unknown                                  && \
    cargo install wasm-gc                                                     && \
    echo 'source $HOME/.cargo/env' >> ~/.bashrc                               && \
    rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git

# install other dependencies for building

RUN apt-get update && apt-get install -q -y \
    software-properties-common \
    cmake \
    pypy \
    pypy-dev

RUN add-apt-repository ppa:git-core/ppa && \
    apt-get update && apt-get install -q -y git

# install dependencies for testing and coverage

RUN apt-get update && apt-get install -q -y \
    lsof \
    procps \
    lcov \
    llvm \
    curl \
    iproute2 \
    python3-pip

RUN apt-get update && apt-get install -q -y \
    libjpeg-dev \
    zlib1g-dev

RUN pip3 install pyopenssl toml cryptography yapf requests Pillow

# install TVM dependencies
RUN apt-get install -q -y \
    llvm-10 \
    clang-10 \
    protobuf-compiler \
    libprotoc-dev \
    libtinfo-dev \
    zlib1g-dev \
    libedit-dev \
    libxml2-dev

# TVM Python builder dependencies
RUN pip3 install onnx==1.9.0 numpy decorator attrs spicy

# Build TVM
RUN git clone https://github.com/apache/tvm /tvm                && \
    cd /tvm                                                     && \
    git checkout 7b3a22e465dd6aca4729504a19beb4bc23312755       && \
    git submodule init                                          && \
    git submodule update                                        && \
    mkdir build                                                 && \
    cd build                                                    && \
    cp ../cmake/config.cmake ./                                 && \
    sed -i '/set(USE_LLVM OFF)/c\set(USE_LLVM ON)' config.cmake && \
    cmake -DUSE_LLVM=ON ..                                      && \
    make -j$(nproc)

# clean up apt caches

RUN apt-get clean && \
    rm -fr /var/lib/apt/lists/* /tmp/* /var/tmp/*
