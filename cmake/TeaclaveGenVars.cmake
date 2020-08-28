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

set(TEACLAVE_PROJECT_ROOT ${PROJECT_SOURCE_DIR})
set(TEACLAVE_BUILD_ROOT ${PROJECT_BINARY_DIR})
set(TEACLAVE_OUT_DIR ${PROJECT_BINARY_DIR}/intermediate)
set(TEACLAVE_INSTALL_DIR ${PROJECT_SOURCE_DIR}/release)
set(TEACLAVE_EDL_DIR ${PROJECT_SOURCE_DIR}/edl)
set(TEACLAVE_SERVICE_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/services)
set(TEACLAVE_EXAMPLE_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/examples)
set(TEACLAVE_BIN_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/bin)
set(TEACLAVE_CLI_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/cli)
set(TEACLAVE_TOOL_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/tool)
set(TEACLAVE_DCAP_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/dcap)
set(TEACLAVE_LIB_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/lib)
set(TEACLAVE_DOC_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/docs)
set(TEACLAVE_TEST_INSTALL_DIR ${TEACLAVE_INSTALL_DIR}/tests)
set(TEACLAVE_AUDITORS_DIR ${TEACLAVE_SERVICE_INSTALL_DIR}/auditors)
set(TEACLAVE_EXAMPLE_AUDITORS_DIR ${TEACLAVE_EXAMPLE_INSTALL_DIR}/auditors)
set(TEACLAVE_TARGET_DIR ${PROJECT_BINARY_DIR}/target)

execute_process(COMMAND bash -c "mktemp -u -d -t teaclave_symlinks.XXXXXXXXXXXX"
                OUTPUT_VARIABLE TEACLAVE_SYMLINKS)
string(STRIP "${TEACLAVE_SYMLINKS}" TEACLAVE_SYMLINKS)
set(TEACLAVE_SYMLINKS ${TEACLAVE_SYMLINKS})

set(THIRD_PARTY_DIR ${PROJECT_SOURCE_DIR}/third_party)
set(UNTRUSTED_TARGET_DIR ${TEACLAVE_TARGET_DIR}/untrusted)
set(UNIX_TARGET_DIR ${TEACLAVE_TARGET_DIR}/unix)
set(TRUSTED_TARGET_DIR ${TEACLAVE_TARGET_DIR}/trusted)
# build.rs will read ENV{ENCLAVE_OUT_DIR} for linking
set(ENCLAVE_OUT_DIR ${TEACLAVE_OUT_DIR})
set(RUST_SGX_SDK ${PROJECT_SOURCE_DIR}/third_party/rust-sgx-sdk)
set(MT_SCRIPT_DIR ${PROJECT_SOURCE_DIR}/cmake/scripts)
set(MT_UNIX_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app)
set(MT_SGXLIB_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_trusted_lib)
set(MT_SGXAPP_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_untrusted_app)

set(SGX_EDGER8R ${SGX_SDK}/bin/x64/sgx_edger8r)
set(SGX_ENCLAVE_SIGNER ${SGX_SDK}/bin/x64/sgx_sign)
set(SGX_LIBRARY_PATH ${SGX_SDK}/lib64)

set(SGX_COMMON_CFLAGS -m64 -O2)
set(SGX_UNTRUSTED_CFLAGS ${SGX_COMMON_CFLAGS} -fPIC -Wno-attributes
                         -I${SGX_SDK}/include -I${RUST_SGX_SDK}/edl)
set(SGX_TRUSTED_CFLAGS
    ${SGX_COMMON_CFLAGS}
    -nostdinc
    -fvisibility=hidden
    -fpie
    -fstack-protector
    -I${RUST_SGX_SDK}/edl
    -I${RUST_SGX_SDK}/common/inc
    -I${SGX_SDK}/include
    -I${SGX_SDK}/include/tlibc
    -I${SGX_SDK}/include/stlport
    -I${SGX_SDK}/include/epid)
join_string("${SGX_COMMON_CFLAGS}" " " STR_SGX_COMMON_CFLAGS)
join_string("${SGX_UNTRUSTED_CFLAGS}" " " STR_SGX_UNTRUSTED_CFLAGS)
join_string("${SGX_TRUSTED_CFLAGS}" " " STR_SGX_TRUSTED_CFLAGS)

if(NOT "${SGX_MODE}" STREQUAL "HW")
  set(Trts_Library_Name sgx_trts_sim)
  set(Service_Library_Name sgx_tservice_sim)
else()
  set(Trts_Library_Name sgx_trts)
  set(Service_Library_Name sgx_tservice)
endif()

set(SGX_ENCLAVE_FEATURES -Z package-features --features mesalock_sgx)
string(TOLOWER "${CMAKE_BUILD_TYPE}" CMAKE_BUILD_TYPE_LOWER)
if(CMAKE_BUILD_TYPE_LOWER STREQUAL "release")
  set(TARGET release)
  set(CARGO_BUILD_FLAGS --release)
else()
  set(TARGET debug)
  set(CARGO_BUILD_FLAGS "")

  if(COV)
    check_exe_dependencies(lcov llvm-cov)
    set(SGX_ENCLAVE_FEATURES -Z package-features --features "mesalock_sgx cov")
    set(CARGO_INCREMENTAL 0)
    set(RUSTFLAGS "${RUSTFLAGS} -D warnings -Zprofile -Ccodegen-units=1 \
-Cllvm_args=-inline-threshold=0 -Coverflow-checks=off")
  endif()
endif()

if(OFFLINE)
  set(EXTRA_CARGO_FLAGS "--offline")
endif()

set(UNIXAPP_PREFIX "unixapp")
set(UNIXLIB_PREFIX "unixlib")
set(SGXAPP_PREFIX "sgxapp")
set(SGXLIB_PREFIX "sgxlib")
set(SGX_MODULES ${SGX_APPS})

# generate SGXLIB_TARGETS (sgxlib-fns sgxlib-kms ...)
new_list_with_prefix(SGXLIB_TARGETS "${SGXLIB_PREFIX}-" ${SGX_MODULES})

set(TEACLAVE_COMMON_ENVS
    TEACLAVE_PROJECT_ROOT=${TEACLAVE_PROJECT_ROOT}
    TEACLAVE_BUILD_ROOT=${TEACLAVE_BUILD_ROOT}
    TEACLAVE_OUT_DIR=${TEACLAVE_OUT_DIR}
    TEACLAVE_SERVICE_INSTALL_DIR=${TEACLAVE_SERVICE_INSTALL_DIR}
    TEACLAVE_EXAMPLE_INSTALL_DIR=${TEACLAVE_EXAMPLE_INSTALL_DIR}
    TEACLAVE_BIN_INSTALL_DIR=${TEACLAVE_BIN_INSTALL_DIR}
    TEACLAVE_CLI_INSTALL_DIR=${TEACLAVE_CLI_INSTALL_DIR}
    TEACLAVE_TOOL_INSTALL_DIR=${TEACLAVE_TOOL_INSTALL_DIR}
    TEACLAVE_DCAP_INSTALL_DIR=${TEACLAVE_DCAP_INSTALL_DIR}
    TEACLAVE_LIB_INSTALL_DIR=${TEACLAVE_LIB_INSTALL_DIR}
    TEACLAVE_DOC_INSTALL_DIR=${TEACLAVE_DOC_INSTALL_DIR}
    TEACLAVE_TEST_INSTALL_DIR=${TEACLAVE_TEST_INSTALL_DIR}
    TEACLAVE_AUDITORS_DIR=${TEACLAVE_AUDITORS_DIR}
    TEACLAVE_EXAMPLE_AUDITORS_DIR=${TEACLAVE_EXAMPLE_AUDITORS_DIR}
    TEACLAVE_TARGET_DIR=${TEACLAVE_TARGET_DIR}
    TEACLAVE_CFG_DIR=${PROJECT_SOURCE_DIR}
    TEACLAVE_BUILD_CFG_DIR=${PROJECT_SOURCE_DIR}
    TEACLAVE_EDL_DIR=${TEACLAVE_EDL_DIR}
    TEACLAVE_SYMLINKS=${TEACLAVE_SYMLINKS}
    SGX_SDK=${SGX_SDK}
    SGX_MODE=${SGX_MODE}
    DCAP=${DCAP}
    ENCLAVE_OUT_DIR=${ENCLAVE_OUT_DIR}
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}
    RUST_SGX_SDK=${RUST_SGX_SDK}
    MT_SCRIPT_DIR=${MT_SCRIPT_DIR}
    MT_SGXAPP_TOML_DIR=${MT_SGXAPP_TOML_DIR}
    CARGO_INCREMENTAL=${CARGO_INCREMENTAL}
    CMAKE_C_COMPILER=${CMAKE_C_COMPILER}
    CC=${MT_SCRIPT_DIR}/cc_wrapper.sh
    MT_RUSTC_WRAPPER=${MT_SCRIPT_DIR}/rustc_wrapper.sh)

set(TARGET_PREP_ENVS
    ${TEACLAVE_COMMON_ENVS}
    CMAKE_SOURCE_DIR=${CMAKE_SOURCE_DIR}
    CMAKE_BINARY_DIR=${CMAKE_BINARY_DIR}
    MESAPY_VERSION=${MESAPY_VERSION}
    SGX_EDGER8R=${SGX_EDGER8R}
    CMAKE_AR=${CMAKE_AR}
    DCAP=${DCAP})

set(TARGET_SGXLIB_ENVS
    ${TEACLAVE_COMMON_ENVS}
    SGX_LIBRARY_PATH=${SGX_LIBRARY_PATH}
    SGX_ENCLAVE_SIGNER=${SGX_ENCLAVE_SIGNER}
    Service_Library_Name=${Service_Library_Name}
    Trts_Library_Name=${Trts_Library_Name}
    TRUSTED_TARGET_DIR=${TRUSTED_TARGET_DIR}
    TARGET=${TARGET})

message("SGX_SDK=${SGX_SDK}")
message("SGX_MODE=${SGX_MODE}")
message("RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}")
message("DCAP=${DCAP}")
message("BUILD_TYPE=${TARGET}")
message("TEACLAVE_SYMLINKS=${TEACLAVE_SYMLINKS}")
