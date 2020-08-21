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

macro(dbg_message)
  if(TEACLAVE_CMAKE_DBG)
    message("${ARGN}")
  endif()
endmacro()

macro(SET_STRVAR_FROM_ENV_OR var default_val docstring)
  if(NOT "$ENV{${var}}" STREQUAL "")
    set(${var}
        "$ENV{${var}}"
        CACHE STRING "${docstring}")
  else()
    set(${var}
        "${default_val}"
        CACHE STRING "${docstring}")
  endif()
endmacro()

function(check_sgx_sdk)
  if(NOT IS_DIRECTORY "${SGX_SDK}")
    message(
      FATAL_ERROR
        "SGX SDK not found at ${SGX_SDK}, please adjust the SGX_SDK env or the CMake config."
    )
  endif()
endfunction()

function(init_submodules)
  if(GIT_FOUND AND EXISTS "${PROJECT_SOURCE_DIR}/.git")
    # Update submodules as needed
    if(GIT_SUBMODULE)
      message(STATUS "Submodule update")
      execute_process(
        COMMAND ${GIT_EXECUTABLE} submodule update --init --recursive
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
        RESULT_VARIABLE GIT_SUBMOD_RESULT)
      if(NOT GIT_SUBMOD_RESULT EQUAL "0")
        message(
          FATAL_ERROR
            "git submodule update --init failed with ${GIT_SUBMOD_RESULT}, please checkout submodules"
        )
      endif()
    endif()
  endif()

  if(NOT EXISTS "${PROJECT_SOURCE_DIR}/third_party/crates-io"
     OR NOT EXISTS "${PROJECT_SOURCE_DIR}/third_party/crates-sgx"
     OR NOT EXISTS "${PROJECT_SOURCE_DIR}/third_party/mesapy"
     OR NOT EXISTS "${PROJECT_SOURCE_DIR}/third_party/rust-sgx-sdk")
    message(
      FATAL_ERROR
        "The submodules were not downloaded! GIT_SUBMODULE was turned off or failed. Please update submodules and try again."
    )
  endif()
endfunction()

macro(rm_trailing_enclave src_str dest_name)
  string(REGEX REPLACE "_enclave$" "" ${dest_name} ${src_str})
endmacro()

# add_cargo_build_target(package_name [TARGET_NAME target_name] # default to
# cg_${package_name} TOML_DIR toml_dir TARGET_DIR target_dir [DEPENDS [dep]...]
# [NOT_SET_COMMON_ENV] [EXTRA_CARGO_FLAGS flg...] )
function(add_cargo_build_target package_name)
  set(options NOT_SET_COMMON_ENV)
  set(oneValueArgs TARGET_NAME TOML_DIR TARGET_DIR INSTALL_DIR
                   EXTRA_CARGO_FLAGS)
  set(multiValueArgs DEPENDS)
  cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}" "${multiValueArgs}"
                        ${ARGN})

  if(DEFINED MTEE_TARGET_NAME)
    set(_target_name ${MTEE_TARGET_NAME})
  else()
    set(_target_name cg_${package_name})
  endif()

  if(DEFINED MTEE_INSTALL_DIR)
    set(_copy_dir ${MTEE_INSTALL_DIR})
  else()
    set(_copy_dir ${TEACLAVE_INSTALL_DIR})
  endif()

  if(MTEE_NOT_SET_COMMON_ENV)
    set(_envs)
  else()
    set(_envs ${TEACLAVE_COMMON_ENVS})
  endif()

  if(DEFINED MTEE_DEPENDS)
    set(_depends DEPENDS ${MTEE_DEPENDS})
  else()
    set(_depends)
  endif()

  add_custom_target(
    ${_target_name} ALL
    COMMAND
      ${CMAKE_COMMAND} -E env ${_envs} RUSTFLAGS=${RUSTFLAGS}
      ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${package_name} --target-dir
      ${MTEE_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS} && cp
      ${MTEE_TARGET_DIR}/${TARGET}/${package_name} ${_copy_dir} ${_depends}
    COMMENT "Building ${_target_name}"
    WORKING_DIRECTORY ${MTEE_TOML_DIR})
endfunction()

# add_cargo_build_dylib_target(package_name [TARGET_NAME target_name] # default to
# cg_${package_name} TOML_DIR toml_dir TARGET_DIR target_dir [DEPENDS [dep]...]
# [NOT_SET_COMMON_ENV] [EXTRA_CARGO_FLAGS flg...] )
function(add_cargo_build_dylib_target package_name)
  set(options NOT_SET_COMMON_ENV)
  set(oneValueArgs TARGET_NAME TOML_DIR TARGET_DIR)
  set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
  cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}" "${multiValueArgs}"
                        ${ARGN})

  if(DEFINED MTEE_TARGET_NAME)
    set(_target_name ${MTEE_TARGET_NAME})
  else()
    set(_target_name cg_${package_name})
  endif()

  if(MTEE_NOT_SET_COMMON_ENV)
    set(_envs)
  else()
    set(_envs ${TEACLAVE_COMMON_ENVS})
  endif()

  if(DEFINED MTEE_DEPENDS)
    set(_depends DEPENDS ${MTEE_DEPENDS})
  else()
    set(_depends)
  endif()

  add_custom_target(
    ${_target_name} ALL
    COMMAND
      ${CMAKE_COMMAND} -E env ${_envs} RUSTFLAGS=${RUSTFLAGS}
      ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${package_name} --target-dir
      ${MTEE_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS} && cp
      ${MTEE_TARGET_DIR}/${TARGET}/lib${package_name}.so
      ${TEACLAVE_LIB_INSTALL_DIR} ${_depends}
    COMMENT "Building ${_target_name} as a dynamic library"
    WORKING_DIRECTORY ${MTEE_TOML_DIR})
endfunction()

# add_sgx_build_target(sgx_lib_path pkg_name [DEPENDS [dep]...] [INSTALL_DIR
# dir] [EXTRA_CARGO_FLAGS flg...] )
function(add_sgx_build_target sgx_lib_path pkg_name)
  set(options)
  set(oneValueArgs INSTALL_DIR EDL_LIB_NAME)
  set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
  cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}" "${multiValueArgs}"
                        ${ARGN})

  if(DEFINED MTEE_DEPENDS)
    set(_depends DEPENDS ${MTEE_DEPENDS})
  else()
    set(_depends)
  endif()

  if(DEFINED MTEE_INSTALL_DIR)
    set(_copy_dir ${MTEE_INSTALL_DIR})
  else()
    set(_copy_dir ${TEACLAVE_INSTALL_DIR})
  endif()

  if(DEFINED MTEE_EDL_LIB_NAME)
    set(_edl_lib_name ${MTEE_EDL_LIB_NAME})
  else()
    set(_edl_lib_name)
  endif()

  rm_trailing_enclave(${pkg_name} pkg_name_no_enclave)

  set(_target_name ${SGXLIB_PREFIX}-${pkg_name_no_enclave})

  if(pkg_name_no_enclave MATCHES "_tests$")
    set(RUSTFLAGS "${RUSTFLAGS} --cfg test_mode")
  endif()

  if(pkg_name_no_enclave MATCHES "_tests$" AND CMAKE_BUILD_TYPE_LOWER STREQUAL
                                               "release")
    set(_enclave_info "/dev/null")
  else()
    set(_enclave_info "${TEACLAVE_OUT_DIR}/${pkg_name}_info.toml")
  endif()

  if(pkg_name_no_enclave MATCHES "_tool$")
    set(_enclave_info "/dev/null")
  endif()

  add_custom_target(
    ${_target_name} ALL
    COMMAND
      ${CMAKE_COMMAND} -E env ${TEACLAVE_COMMON_ENVS} RUSTFLAGS=${RUSTFLAGS}
      ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${pkg_name} --target-dir
      ${TRUSTED_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${SGX_ENCLAVE_FEATURES}
      ${MTEE_EXTRA_CARGO_FLAGS}
    COMMAND
      ${CMAKE_COMMAND} -E env ${TARGET_SGXLIB_ENVS}
      SGX_COMMON_CFLAGS=${STR_SGX_COMMON_CFLAGS} CUR_PKG_NAME=${pkg_name}
      CUR_PKG_PATH=${sgx_lib_path} CUR_INSTALL_DIR=${_copy_dir}
      ${MT_SCRIPT_DIR}/sgx_link_sign.sh ${_edl_lib_name} ${_depends}
    COMMAND
      cat ${TEACLAVE_OUT_DIR}/${pkg_name}.meta.txt | python
      ${MT_SCRIPT_DIR}/gen_enclave_info_toml.py ${pkg_name_no_enclave} >
      ${_enclave_info}
    COMMENT "Building ${_target_name}, enclave info to ${_enclave_info}"
    WORKING_DIRECTORY ${MT_SGXLIB_TOML_DIR})
endfunction()

function(add_enclave_sig_target_n_hooks)
  # add a target to generate enclave sig files
  add_custom_target(
    update_sig ALL
    COMMAND ${TEACLAVE_COMMON_ENVS} ${MT_SCRIPT_DIR}/gen_enclave_sig.sh
    COMMENT "Generating enclave signatures..."
    DEPENDS ${SGXLIB_TARGETS})

  # Hook the convenience targets for SGX modules so manually `make kms/tms/...`
  # will trigger updating enclave sig files
  foreach(sgx_module ${SGX_MODULES})
    add_custom_command(
      TARGET ${sgx_module}
      POST_BUILD
      COMMENT "Updating enclave signatures..."
      COMMAND ${TEACLAVE_COMMON_ENVS} ${MT_SCRIPT_DIR}/gen_enclave_sig.sh)
  endforeach()
endfunction()

function(join_string values glue out)
  string(REGEX REPLACE "([^\\]|^);" "\\1${glue}" _res "${values}")
  string(REGEX REPLACE "[\\](.)" "\\1" _res "${_res}")
  set(${out}
      "${_res}"
      PARENT_SCOPE)
endfunction()

function(generate_env_file)
  set(envs ${TEACLAVE_COMMON_ENVS})
  list(FILTER envs INCLUDE REGEX "TEACLAVE_PROJECT_ROOT|TEACLAVE_CFG_DIR|\
TEACLAVE_BUILD_CFG_DIR|TEACLAVE_OUT_DIR|TEACLAVE_AUDITORS_DIR")
  # add extra env vars
  list(APPEND envs "RUST_LOG=info" "RUST_BACKTRACE=1")
  join_string("${envs}" "\nexport " env_file)
  string(PREPEND env_file "export ")
  string(APPEND env_file "\n")
  file(WRITE ${PROJECT_BINARY_DIR}/environment ${env_file})
  message(STATUS "====== ${PROJECT_BINARY_DIR}/environment GENERATED ======")
endfunction()

function(gen_convenience_targets)
  # add a target with the same name for each unix_module
  foreach(unix_module ${UNIX_APPS})
    add_custom_target(${unix_module} DEPENDS ${UNIXAPP_PREFIX}-${unix_module})
  endforeach()

  # add a target with the same name for each sgx_module (build sgxlib+sgxapp)
  foreach(sgx_module ${SGX_MODULES})
    add_custom_target(${sgx_module} DEPENDS ${SGXAPP_PREFIX}-${sgx_module}
                                            ${SGXLIB_PREFIX}-${sgx_module})
  endforeach()
endfunction()

function(new_list_with_prefix new_list_name prefix)
  set(_new_list)
  foreach(item ${ARGN})
    string(PREPEND item ${prefix})
    set(_new_list ${_new_list} ${item})
  endforeach()
  set(${new_list_name}
      ${_new_list}
      PARENT_SCOPE)
endfunction()

function(check_exe_dependencies)
  foreach(exe ${ARGN})
    execute_process(COMMAND bash -c "type ${exe}" OUTPUT_QUIET ERROR_QUIET
                    RESULT_VARIABLE _res)
    if(_res)
      message(
        FATAL_ERROR
          "Teaclave depends on \"${exe}\" but the command was not found. \
Please install the dependency and retry.")
    endif()
  endforeach()
endfunction()

function(parse_cargo_packages pkg_names)
  set(options)
  set(oneValueArgs CARGO_TOML_PATH PKG_PATHS CATEGORIES EDL_NAMES)
  set(multiValueArgs)

  cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}" "${multiValueArgs}"
                        ${ARGN})

  set(_output)
  set(err)

  execute_process(
    COMMAND python ${PROJECT_SOURCE_DIR}/cmake/scripts/parse_cargo_packages.py
            ${MTEE_CARGO_TOML_PATH} ${PROJECT_SOURCE_DIR}
    OUTPUT_VARIABLE _output
    ERROR_VARIABLE err)

  if(NOT (err STREQUAL ""))
    message(FATAL_ERROR "failed to load packages: ${err}")
  endif()

  string(REGEX REPLACE "\n" ";" _out_list ${_output})
  list(LENGTH _out_list LLEN)

  if(DEFINED MTEE_EDL_NAMES)
    list(GET _out_list 3 _edl_names)
    string(REPLACE ":" ";" _edl_names ${_edl_names})
    set(${MTEE_EDL_NAMES}
        ${_edl_names}
        PARENT_SCOPE)
    dbg_message("${MTEE_EDL_NAMES}=${_edl_names}\n")
  endif()

  if(DEFINED MTEE_CATEGORIES)
    list(GET _out_list 2 _categories)
    string(REPLACE ":" ";" _categories ${_categories})
    set(${MTEE_CATEGORIES}
        ${_categories}
        PARENT_SCOPE)
    dbg_message("${MTEE_CATEGORIES}=${_categories}\n")
  endif()

  if(DEFINED MTEE_PKG_PATHS)
    list(GET _out_list 1 _pkg_paths)
    string(REPLACE ":" ";" _pkg_paths ${_pkg_paths})
    set(${MTEE_PKG_PATHS}
        ${_pkg_paths}
        PARENT_SCOPE)
    dbg_message("${MTEE_PKG_PATHS}=${_pkg_paths}\n")
  endif()

  # level up the local variable to its parent scope
  list(GET _out_list 0 _pkg_names)
  string(REPLACE ":" ";" _pkg_names ${_pkg_names})
  set(${pkg_names}
      ${_pkg_names}
      PARENT_SCOPE)
  dbg_message("${pkg_names}=${_pkg_names}\n")
endfunction()
