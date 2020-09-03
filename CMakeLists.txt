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
find_package(Git)
find_package(OpenSSL)
check_exe_dependencies(rustup pypy)
pkg_check_modules(INTEL_SGX_SDK QUIET libsgx_urts=2.9.101.2)
if(NOT INTEL_SGX_SDK_FOUND)
  message(FATAL_ERROR "Intel SGX SDK (version 2.9.1) is not found or version not matched.")
endif()

# ====== VARIABLES FOR CMAKE -D{VAR}=VAL CONFIGURATION BEGIN ======
set_strvar_from_env_or(SGX_SDK "/opt/sgxsdk" "Path of SGX SDK")
set_strvar_from_env_or(RUSTFLAGS "" "Rust flags")
set_strvar_from_env_or(TEACLAVE_CMAKE_DBG ""
                       "set to turn on debug message for cmake")
set(MESAPY_VERSION 947fb3f598eede83ba0e33b5b5655b9a9597c2d8)
set(RUSTUP_TOOLCHAIN "nightly-2020-04-07")
option(COV "Turn on/off coverage" OFF)
option(OFFLINE "Turn on/off cargo offline" ON)
option(TEST_MODE "Turn on/off test mode" OFF)
option(SGX_SIM_MODE "Turn on/off sgx simulation mode" OFF)
option(DCAP "Turn on/off DCAP attestation" OFF)
option(GIT_SUBMODULE "Check submodules during build" ON)
option(USE_PREBUILT_MESAPY "Use prebuilt MesaPy SGX executor" ON)
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

include(TeaclaveGenVars)

# ========== CONFIG-TIME CHECKS AND INITIALIZATIONS BEGIN =========
check_sgx_sdk()
generate_env_file()
# =========== CONFIG-TIME CHECKS AND INITIALIZATIONS END ==========

# UTIL TARGETS: prep, sgx-test, cov, cov-clean, format, check, doc, etc.
include(UtilTargets)

# unix_app
list(LENGTH UNIX_APPS UNIX_APP_LEN)
math(EXPR UNIX_APP_LAST_INDEX "${UNIX_APP_LEN} - 1")
foreach(_i RANGE ${UNIX_APP_LAST_INDEX})
  list(GET UNIX_APPS ${_i} _pkg_name)
  list(GET UNIX_APP_CATEGORIES ${_i} _category)
  if(${_category} STREQUAL "dcap" AND NOT DCAP)
    continue()
  endif()
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
    prep)
endforeach()

# sgx_untrusted_app
list(LENGTH SGX_APPS SGX_APP_LEN)
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
    prep)
endforeach()

set(MESAPY_OUTPUTS
  ${TEACLAVE_OUT_DIR}/libpypy-c.a
  ${TEACLAVE_OUT_DIR}/libffi.a
  ${TEACLAVE_OUT_DIR}/libsgx_tlibc_ext.a
  ${TEACLAVE_OUT_DIR}/libsgx_ulibc.a
  ${TEACLAVE_OUT_DIR}/ffi.o
)
if(USE_PREBUILT_MESAPY)
add_custom_command(
  OUTPUT ${MESAPY_OUTPUTS}
  COMMAND
    wget -qN https://mesapy.org/release/${MESAPY_VERSION}-mesapy-sgx.tar.gz &&
    tar xzf ${MESAPY_VERSION}-mesapy-sgx.tar.gz
  WORKING_DIRECTORY ${TEACLAVE_OUT_DIR})
else()
  add_custom_command(
    OUTPUT ${MESAPY_OUTPUTS}
    COMMAND
      make sgx &&
      make -C sgx/examples/exec &&
      cp pypy/goal/libpypy-c.a
        sgx/libffi/build_dir/lib/libffi.a
        sgx/sgx_tlibc_ext/libsgx_tlibc_ext.a
        sgx/sgx_ulibc/libsgx_ulibc.a
        sgx/examples/exec/Enclave/src/ffi.o
        ${TEACLAVE_OUT_DIR}
    WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}/third_party/mesapy)
endif()
add_custom_target(mesapy
  DEPENDS ${MESAPY_OUTPUTS}
  )

# mesapy components
add_custom_command(
  OUTPUT ${TEACLAVE_OUT_DIR}/acs_py_enclave.c
  COMMAND
    env ARGS PYTHONPATH=${PROJECT_SOURCE_DIR}/third_party/mesapy/sgx
    PYPY_FFI_OUTDIR=${TEACLAVE_OUT_DIR} pypy
    ${PROJECT_SOURCE_DIR}/services/access_control/python/ffi.py
  DEPENDS prep ${PROJECT_SOURCE_DIR}/services/access_control/python/ffi.py
          ${PROJECT_SOURCE_DIR}/services/access_control/python/acs_engine.py
  COMMENT "Generating mesapy ffi stubs")
add_library(pycomponent STATIC ${TEACLAVE_OUT_DIR}/acs_py_enclave.c)
set_target_properties(pycomponent PROPERTIES ARCHIVE_OUTPUT_DIRECTORY
                                             ${TEACLAVE_OUT_DIR})
target_compile_definitions(pycomponent PUBLIC SGX)
if(NOT EXISTS "/usr/lib/pypy/include")
  message(
    FATAL_ERROR
      "pypy development package not found\nFor Ubuntu, please run `apt-get install pypy-dev`"
  )
endif()
target_compile_options(pycomponent PUBLIC -UWITH_THREAD -O2 -fPIC -Wimplicit
                                          -I/usr/lib/pypy/include)

# sgx_trusted_lib
list(LENGTH SGX_LIBS SGX_LIB_LEN)
math(EXPR SGX_LIB_LAST_INDEX "${SGX_LIB_LEN} - 1")
foreach(_i RANGE ${SGX_LIB_LAST_INDEX})
  list(GET SGX_LIBS ${_i} _pkg_name)
  list(GET SGX_LIB_PATHS ${_i} _pkg_path)
  list(GET SGX_LIB_CATEGORIES ${_i} _category)
  list(GET EDL_LIB_NAMES ${_i} _edl_lib_name)
  add_sgx_build_target(
    ${_pkg_path}
    ${_pkg_name}
    DEPENDS
    prep
    mesapy
    pycomponent
    INSTALL_DIR
    ${TEACLAVE_INSTALL_DIR}/${_category}
    EDL_LIB_NAME
    ${_edl_lib_name})
endforeach()

# Teaclave C SDK add_cargo_build_dylib_target(teaclave_sdk_c TARGET_NAME
# "${UNIXLIB_PREFIX}-teaclave_sdk_c" TOML_DIR ${MT_UNIX_TOML_DIR} TARGET_DIR
# ${UNIX_TARGET_DIR} DEPENDS prep )

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
