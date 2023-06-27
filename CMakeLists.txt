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

cmake_minimum_required(VERSION 3.10)
project(Teaclave C)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(TeaclaveUtils)
include(ExternalProject)
find_package(PkgConfig QUIET)
if (NOT PkgConfig_FOUND)
  message(FATAL_ERROR "PkgConfig is not found.")
endif()
find_package(Git)
find_package(OpenSSL)
check_exe_dependencies(rustup)
pkg_check_modules(INTEL_SGX_SDK QUIET libsgx_urts>=2.17.101.1)
if(NOT INTEL_SGX_SDK_FOUND)
  message(FATAL_ERROR "Intel SGX SDK (version 2.17.1) is not found or version not matched.")
endif()

# ====== VARIABLES FOR CMAKE -D{VAR}=VAL CONFIGURATION BEGIN ======
set_strvar_from_env_or(SGX_SDK "/opt/sgxsdk" "Path of SGX SDK")
set_strvar_from_env_or(RUSTFLAGS "" "Rust flags")
set_strvar_from_env_or(TEACLAVE_CMAKE_DBG ""
                       "set to turn on debug message for cmake")
set(MESAPY_VERSION v1.0.0-sgx)
set(RUSTUP_TOOLCHAIN "nightly-2022-10-22")
option(COV "Turn on/off coverage" OFF)
option(OFFLINE "Turn on/off cargo offline" OFF)
option(TEST_MODE "Turn on/off test mode" OFF)
option(SGX_SIM_MODE "Turn on/off sgx simulation mode" OFF)
option(DCAP "Turn on/off DCAP attestation" OFF)
option(GIT_SUBMODULE "Check submodules during build" ON)
option(EXECUTOR_BUILTIN "Compile with the Builtin executor" ON)
option(EXECUTOR_MESAPY "Compile with the MesaPy executor" OFF)
option(EXECUTOR_WAMR "Compile with the WebAssembly Micro Runtime executor" ON)
init_submodules()

if(DCAP)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg dcap")
endif()

if(TEST_MODE)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg test_mode")
endif()

if(SGX_SIM_MODE)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg sgx_sim")
  set(SGX_MODE "SW")
else()
  set(SGX_MODE "HW")
endif()

if(EXECUTOR_BUILTIN)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg executor_builtin")
endif()

if(EXECUTOR_MESAPY)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg executor_mesapy")
endif()

if(EXECUTOR_WAMR)
  set(RUSTFLAGS "${RUSTFLAGS} --cfg executor_wamr")
endif()

# ======= VARIABLES FOR CMAKE -D{VAR}=VAL CONFIGURATION END =======

# =============== VARIABLES FOR MANUAL CHANGE BEGIN ===============
set(UNIX_LIBS teaclave_sdk protected_fs_rs)
# ================ VARIABLES FOR MANUAL CHANGE END ================

# UNIX_APPS, SGX_APPS and SGX_LIBS are parsed from corresponding toml files
parse_cargo_packages(
  UNIX_APPS CARGO_TOML_PATH
  ${PROJECT_SOURCE_DIR}/cmake/tomls/Cargo.unix_app.toml CATEGORIES
  UNIX_APP_CATEGORIES)

parse_cargo_packages(
  SGX_APPS CARGO_TOML_PATH
  ${PROJECT_SOURCE_DIR}/cmake/tomls/Cargo.sgx_untrusted_app.toml CATEGORIES
  SGX_APP_CATEGORIES)

parse_cargo_packages(
  SGX_LIBS
  CARGO_TOML_PATH
  ${PROJECT_SOURCE_DIR}/cmake/tomls/Cargo.sgx_trusted_lib.toml
  CATEGORIES
  SGX_LIB_CATEGORIES
  PKG_PATHS
  SGX_LIB_PATHS
  EDL_NAMES
  EDL_LIB_NAMES)

if(NOT TEST_MODE)
  list(FILTER SGX_APPS EXCLUDE REGEX "_tests$")
  list(FILTER SGX_APP_CATEGORIES EXCLUDE REGEX "tests")
  list(FILTER SGX_LIBS EXCLUDE REGEX "_tests_enclave$")
  list(FILTER SGX_LIB_CATEGORIES EXCLUDE REGEX "tests")
endif()

if(NOT DCAP)
  list(FILTER UNIX_APPS EXCLUDE REGEX "dcap")
  list(FILTER UNIX_APP_CATEGORIES EXCLUDE REGEX "dcap")
endif()

include(TeaclaveGenVars)

# ========== CONFIG-TIME CHECKS AND INITIALIZATIONS BEGIN =========
check_sgx_sdk()
generate_env_file()
# =========== CONFIG-TIME CHECKS AND INITIALIZATIONS END ==========

# UTIL TARGETS: prep, sgx-test, cov, cov-clean, format, check, doc, etc.
include(UtilTargets)

# unix_app
list(LENGTH UNIX_APPS UNIX_APP_LEN)
set(UNIX_APPS_DEPENDS prep)
math(EXPR UNIX_APP_LAST_INDEX "${UNIX_APP_LEN} - 1")
foreach(_i RANGE ${UNIX_APP_LAST_INDEX})
  list(GET UNIX_APPS ${_i} _pkg_name)
  list(GET UNIX_APP_CATEGORIES ${_i} _category)
  add_cargo_build_target(
    ${_pkg_name}
    TARGET_NAME
    "${UNIXAPP_PREFIX}-${_pkg_name}"
    TOML_DIR
    ${MT_UNIX_TOML_DIR}
    TARGET_DIR
    ${UNIX_TARGET_DIR}
    INSTALL_DIR
    ${TEACLAVE_INSTALL_DIR}/${_category}
    EXTRA_CARGO_FLAGS
    ${EXTRA_CARGO_FLAGS}
    DEPENDS
    ${UNIX_APPS_DEPENDS})
endforeach()

# sgx_untrusted_app
list(LENGTH SGX_APPS SGX_APP_LEN)
set(SGX_APPS_DEPENDS prep)
math(EXPR SGX_APP_LAST_INDEX "${SGX_APP_LEN} - 1")
foreach(_i RANGE ${SGX_APP_LAST_INDEX})
  list(GET SGX_APPS ${_i} _pkg_name)
  list(GET SGX_APP_CATEGORIES ${_i} _category)
  add_cargo_build_target(
    ${_pkg_name}
    TARGET_NAME
    "${SGXAPP_PREFIX}-${_pkg_name}"
    TOML_DIR
    ${MT_SGXAPP_TOML_DIR}
    TARGET_DIR
    ${UNTRUSTED_TARGET_DIR}
    INSTALL_DIR
    ${TEACLAVE_INSTALL_DIR}/${_category}
    EXTRA_CARGO_FLAGS
    ${EXTRA_CARGO_FLAGS}
    DEPENDS
    ${SGX_APPS_DEPENDS})
endforeach()

# WAMR library

set(WAMR_OUTPUTS
  ${TEACLAVE_OUT_DIR}/libvmlib.a
)

ExternalProject_Add(wamr_teaclave
  SOURCE_DIR ${WAMR_TEACLAVE_ROOT_DIR}
  BINARY_DIR ${WAMR_TEACLAVE_ROOT_DIR}/build
  INSTALL_COMMAND ""
  LOG_BUILD 1
)

add_custom_command(
  OUTPUT ${WAMR_OUTPUTS}
  DEPENDS wamr_teaclave
  COMMAND
    cp ${WAMR_TEACLAVE_ROOT_DIR}/build/libvmlib.a ${TEACLAVE_OUT_DIR}
  WORKING_DIRECTORY ${WAMR_TEACLAVE_ROOT_DIR}/build
  )

add_custom_target(wamr
  DEPENDS ${WAMR_TEACLAVE_ROOT_DIR}/CMakeLists.txt ${WAMR_OUTPUTS}
  )

# std Aware Cargo
set(DEFAULT_STD_PATH ${TRUSTED_TARGET_DIR}/default_std/sysroot/lib/rustlib/${SGX_LIB_TARGET}/lib)
set(COV_STD_PATH ${TRUSTED_TARGET_DIR}/cov_std/sysroot/lib/rustlib/${SGX_LIB_TARGET}/lib)

add_custom_target(
    default_std
    COMMAND
      ${CMAKE_COMMAND} -E env ${TEACLAVE_COMMON_ENVS} RUSTFLAGS=""
      cargo build ${CARGO_BUILD_FLAGS} --target-dir ${TRUSTED_TARGET_DIR}/default_std
      --features backtrace,capi,env,net,thread,untrusted_time,unsupported_process -Zbuild-std=core,alloc
      --target ${RUST_SGX_SDK}/rustlib/${SGX_LIB_TARGET}.json
    COMMAND
    mkdir -p ${DEFAULT_STD_PATH}
    COMMAND
      cp -r ${TRUSTED_TARGET_DIR}/default_std/${SGX_LIB_TARGET}/${TARGET}/deps/*
      ${DEFAULT_STD_PATH}
    DEPENDS prep
    COMMENT "Building default std"
    WORKING_DIRECTORY ${RUST_SGX_SDK}/rustlib/std
    ) # default_std

add_custom_target(
    cov_std
    COMMAND
      ${CMAKE_COMMAND} -E env ${TEACLAVE_COMMON_ENVS} RUSTFLAGS=""
      cargo build ${CARGO_BUILD_FLAGS} --target-dir ${TRUSTED_TARGET_DIR}/cov_std
      --features backtrace,capi,env,net,profiler,thread,untrusted_time,unsupported_process -Zbuild-std=core,alloc
      --target ${RUST_SGX_SDK}/rustlib/${SGX_LIB_TARGET}.json
    COMMAND
    mkdir -p ${COV_STD_PATH}
    COMMAND
      cp -r ${TRUSTED_TARGET_DIR}/cov_std/${SGX_LIB_TARGET}/${TARGET}/deps/*
      ${COV_STD_PATH}
    DEPENDS prep
    COMMENT "Building cov std"
    WORKING_DIRECTORY ${RUST_SGX_SDK}/rustlib/std
    ) # cov_std

# sgx_trusted_lib
list(LENGTH SGX_LIBS SGX_LIB_LEN)
set(SGX_LIB_DEPENDS prep)
math(EXPR SGX_LIB_LAST_INDEX "${SGX_LIB_LEN} - 1")
foreach(_i RANGE ${SGX_LIB_LAST_INDEX})
  list(GET SGX_LIBS ${_i} _pkg_name)
  list(GET SGX_LIB_PATHS ${_i} _pkg_path)
  list(GET SGX_LIB_CATEGORIES ${_i} _category)
  list(GET EDL_LIB_NAMES ${_i} _edl_lib_name)

  if(EXECUTOR_WAMR)
    list(APPEND SGX_LIB_DEPENDS wamr)
  endif()

  if(COV)
    list(APPEND SGX_LIB_DEPENDS cov_std)
  else()
    list(APPEND SGX_LIB_DEPENDS default_std)
  endif()

  add_sgx_build_target(
    ${_pkg_path}
    ${_pkg_name}
    DEPENDS
    ${SGX_LIB_DEPENDS}
    INSTALL_DIR
    ${TEACLAVE_INSTALL_DIR}/${_category}
    EDL_LIB_NAME
    ${_edl_lib_name})
endforeach()

# Dylib/staticlib of Teaclave Rust Client SDK
add_cargo_build_dylib_staticlib_target(teaclave_client_sdk
  TARGET_NAME "teaclave_client_sdk"
  TOML_DIR ${MT_UNIX_TOML_DIR}
  TARGET_DIR ${UNIX_TARGET_DIR}
  DEPENDS prep)

# example/quickstart_c link_directories(${TEACLAVE_LIB_INSTALL_DIR})
# add_executable(quickstart_c
# ${TEACLAVE_PROJECT_ROOT}/examples/quickstart_c/main.c)
# add_dependencies(quickstart_c prep ${UNIXLIB_PREFIX}-teaclave_sdk_c)
# target_include_directories(quickstart_c PUBLIC
# ${TEACLAVE_PROJECT_ROOT}/teaclave_sdk/c_sdk/include/)
# target_link_libraries(quickstart_c teaclave_sdk_c) add_custom_command(TARGET
# quickstart_c POST_BUILD COMMAND ${CMAKE_COMMAND} -E copy
# ${CMAKE_CURRENT_BINARY_DIR}/quickstart_c
# ${TEACLAVE_EXAMPLE_INSTALL_DIR}/quickstart_c )

add_enclave_sig_target_n_hooks()

set(LIBOS_EXTRA_CARGO_FLAGS --features "libos")
set(LIBOS_DEPENDS prep)
if(EXECUTOR_WAMR)
    list(APPEND LIBOS_DEPENDS wamr)
endif()
add_cargo_build_target(
    teaclave_execution_service
    TARGET_NAME
    "teaclave_execution_service_libos"
    TOML_DIR
    ${MT_SGXAPP_TOML_DIR}
    TARGET_DIR
    ${UNTRUSTED_TARGET_DIR}
    INSTALL_DIR
    ${TEACLAVE_BIN_INSTALL_DIR}/teaclave_execution_service_libos
    EXTRA_CARGO_FLAGS
    ${LIBOS_EXTRA_CARGO_FLAGS}
    DEPENDS
    ${LIBOS_DEPENDS})
