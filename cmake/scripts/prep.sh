#!/bin/bash

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

set -e
REQUIRED_ENVS=("CMAKE_SOURCE_DIR" "CMAKE_BINARY_DIR"
"TEACLAVE_OUT_DIR" "TEACLAVE_TARGET_DIR" "RUSTUP_TOOLCHAIN" "MESAPY_VERSION"
"SGX_EDGER8R" "TEACLAVE_EDL_DIR" "SGX_SDK" "RUST_SGX_SDK" "CMAKE_C_COMPILER"
"CMAKE_AR" "SGX_UNTRUSTED_CFLAGS" "SGX_TRUSTED_CFLAGS" "MT_SCRIPT_DIR"
"TEACLAVE_SERVICE_INSTALL_DIR" "TEACLAVE_EXAMPLE_INSTALL_DIR" "TEACLAVE_BIN_INSTALL_DIR"
"TEACLAVE_CLI_INSTALL_DIR" "TEACLAVE_TOOL_INSTALL_DIR" "TEACLAVE_DCAP_INSTALL_DIR"
"TEACLAVE_LIB_INSTALL_DIR" "TEACLAVE_TEST_INSTALL_DIR"
"TEACLAVE_AUDITORS_DIR" "TEACLAVE_EXAMPLE_AUDITORS_DIR" "DCAP" "TEACLAVE_SYMLINKS"
"TEACLAVE_PROJECT_ROOT"
)

for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

${MT_SCRIPT_DIR}/setup_cmake_tomls.py ${CMAKE_SOURCE_DIR} ${CMAKE_BINARY_DIR}
mkdir -p ${TEACLAVE_OUT_DIR} ${TEACLAVE_TARGET_DIR} ${TEACLAVE_SERVICE_INSTALL_DIR} \
      ${TEACLAVE_EXAMPLE_INSTALL_DIR} ${TEACLAVE_CLI_INSTALL_DIR} ${TEACLAVE_TOOL_INSTALL_DIR} \
      ${TEACLAVE_BIN_INSTALL_DIR} ${TEACLAVE_LIB_INSTALL_DIR} \
    ${TEACLAVE_TEST_INSTALL_DIR} ${TEACLAVE_AUDITORS_DIR} ${TEACLAVE_EXAMPLE_AUDITORS_DIR}
if [ -n "$DCAP" ]; then
    mkdir -p ${TEACLAVE_DCAP_INSTALL_DIR}
    cp ${CMAKE_SOURCE_DIR}/dcap/Rocket.toml ${TEACLAVE_DCAP_INSTALL_DIR}/Rocket.toml
    cp ${CMAKE_SOURCE_DIR}/keys/dcap_server_cert.pem ${TEACLAVE_DCAP_INSTALL_DIR}/
    cp ${CMAKE_SOURCE_DIR}/keys/dcap_server_key.pem ${TEACLAVE_DCAP_INSTALL_DIR}/
fi
# copy auditors to install directory to make it easy to package all built things
cp -RT ${CMAKE_SOURCE_DIR}/keys/auditors/ ${TEACLAVE_AUDITORS_DIR}/
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${TEACLAVE_SERVICE_INSTALL_DIR}
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${TEACLAVE_TEST_INSTALL_DIR}
cp -r ${CMAKE_SOURCE_DIR}/tests/fixtures/ ${TEACLAVE_TEST_INSTALL_DIR}
ln -f -s ${TEACLAVE_TEST_INSTALL_DIR}/fixtures ${TEACLAVE_SERVICE_INSTALL_DIR}/fixtures
cp -r ${CMAKE_SOURCE_DIR}/tests/scripts/ ${TEACLAVE_TEST_INSTALL_DIR}
# create the following symlinks to make remapped paths accessible and avoid repeated building
mkdir -p ${TEACLAVE_SYMLINKS}
ln -snf ${HOME}/.cargo ${TEACLAVE_SYMLINKS}/cargo_home
ln -snf ${CMAKE_SOURCE_DIR} ${TEACLAVE_SYMLINKS}/teaclave_src
ln -snf ${CMAKE_BINARY_DIR} ${TEACLAVE_SYMLINKS}/teaclave_build
# cleanup sgx_unwind/libunwind
(cd ${CMAKE_SOURCE_DIR}/third_party/crates-sgx/ && git clean -fdx vendor/sgx_unwind/libunwind/)
if git submodule status | egrep -q '^[-]|^[+]'; then echo 'INFO: Need to reinitialize git submodules' && git submodule update --init --recursive; fi
rustup install --no-self-update ${RUSTUP_TOOLCHAIN} > /dev/null 2>&1

# build edl_libs
function build_edl() {
    echo 'INFO: Start to build EDL.'

    cd ${TEACLAVE_OUT_DIR}
    for edl in ${TEACLAVE_EDL_DIR}/*.edl
    do
        # $FILE_NAME.edl to $FILE_NAME_t.c
        ${SGX_EDGER8R} --trusted ${edl} --search-path ${SGX_SDK}/include \
            --search-path ${RUST_SGX_SDK}/edl --search-path ${TEACLAVE_PROJECT_ROOT}/edl \
            --trusted-dir ${TEACLAVE_OUT_DIR}

        # $FILE_NAME.edl to $FILE_NAME_u.c
        ${SGX_EDGER8R} --untrusted ${edl} --search-path ${SGX_SDK}/include \
            --search-path ${RUST_SGX_SDK}/edl --search-path ${TEACLAVE_PROJECT_ROOT}/edl \
            --untrusted-dir ${TEACLAVE_OUT_DIR}

        fname=$(basename "$edl" .edl)

        # $FILE_NAME_u.c -> lib$FILE_NAME_u.o -> lib$FILE_NAME_u.a
        ${CMAKE_C_COMPILER} ${SGX_UNTRUSTED_CFLAGS} -c "${fname}_u.c" -o "lib${fname}_u.o"
        ${CMAKE_AR} rcsD "lib${fname}_u.a" "lib${fname}_u.o"

        # $FILE_NAME_t.c to $FILE_NAME_t.o
        ${CMAKE_C_COMPILER} ${SGX_TRUSTED_CFLAGS} -c "${fname}_t.c" -o "lib${fname}_t.o"
    done
}

# check
for edl in ${TEACLAVE_EDL_DIR}/*.edl
do
    fname=$(basename "${edl}" .edl)
    tlib="${TEACLAVE_OUT_DIR}/lib${fname}_t.o"
    ulib="${TEACLAVE_OUT_DIR}/lib${fname}_u.a"

    if [[ ! -f "${tlib}" ]] ||  [[ ! -f "${ulib}" ]] ; then
        build_edl
        break
    fi
done
