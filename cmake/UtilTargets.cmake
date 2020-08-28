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

add_custom_target(
  prep ALL
  COMMAND
    ${CMAKE_COMMAND} -E env ${TARGET_PREP_ENVS}
    SGX_UNTRUSTED_CFLAGS=${STR_SGX_UNTRUSTED_CFLAGS}
    SGX_TRUSTED_CFLAGS=${STR_SGX_TRUSTED_CFLAGS} ${MT_SCRIPT_DIR}/prep.sh)

add_custom_target(
  format
  COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
  COMMAND
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${TEACLAVE_PROJECT_ROOT}
    -path ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o
    -path ${TEACLAVE_PROJECT_ROOT}/.git -prune -o
    -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.rs" -exec rustfmt {} +
  COMMAND
    find ${TEACLAVE_PROJECT_ROOT}
    -path ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o
    -path ${TEACLAVE_PROJECT_ROOT}/.git -prune -o
    -path ${TEACLAVE_PROJECT_ROOT}/services/access_control -prune -o
    -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.py" -exec yapf -i {} +
  COMMENT "Formating every .rs and .py file with rustfmt and yapf"
  DEPENDS prep)

add_custom_target(
  check
  COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
  COMMAND
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${TEACLAVE_PROJECT_ROOT} -path
    ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o -path
    ${TEACLAVE_PROJECT_ROOT}/.git -prune -o -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.rs" -exec rustfmt --check {} +
  COMMAND
    find ${TEACLAVE_PROJECT_ROOT}
    -path ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o
    -path ${TEACLAVE_PROJECT_ROOT}/.git -prune -o
    -path ${TEACLAVE_PROJECT_ROOT}/services/access_control -prune -o
    -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.py" -exec yapf -d {} +
  COMMENT "Checking the format of every .rs and .py file with rustfmt and yapf"
  DEPENDS prep)

if(TEST_MODE)
  add_custom_target(run-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                      ${MT_SCRIPT_DIR}/test.sh)
  add_custom_target(run-unit-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                           ${MT_SCRIPT_DIR}/test.sh unit)
  add_custom_target(
    run-integration-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                  ${MT_SCRIPT_DIR}/test.sh integration)
  add_custom_target(
    run-functional-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                 ${MT_SCRIPT_DIR}/test.sh functional)
else()
  add_custom_target(
    run-tests
    COMMAND
      echo
      "Note: Testing is not enabled in this build. Run cmake again with -DTEST_MODE=ON"
  )
endif()

add_custom_target(run-examples COMMAND ${TEACLAVE_COMMON_ENVS}
  ${MT_SCRIPT_DIR}/test.sh example)

add_custom_target(cov COMMAND ${TEACLAVE_COMMON_ENVS}
                              ${MT_SCRIPT_DIR}/gen_cov.sh)

add_custom_target(
  cov-clean
  COMMAND rm -rf ${TEACLAVE_OUT_DIR}/*.info ${TEACLAVE_OUT_DIR}/cov_* cov.info
          cov_report
  COMMAND find . -name *.gcda -exec rm {} "\;"
  WORKING_DIRECTORY ${PROJECT_SOURCE_DIR})

# add folders for "clean" target
set_property(
  DIRECTORY
  PROPERTY ADDITIONAL_MAKE_CLEAN_FILES "${TEACLAVE_INSTALL_DIR}"
           "${TEACLAVE_OUT_DIR}" "${TEACLAVE_TARGET_DIR}"
           "${PROJECT_BINARY_DIR}/cmake_tomls")

# doc target
add_custom_target(
  doc
  COMMAND
    make DOC=1 all
    && mkdir -p ${TEACLAVE_DOC_INSTALL_DIR}
    && cp -RT ${TEACLAVE_TARGET_DIR}/trusted/doc ${TEACLAVE_DOC_INSTALL_DIR}/enclave
    && cp -RT ${TEACLAVE_TARGET_DIR}/untrusted/doc ${TEACLAVE_DOC_INSTALL_DIR}/app
    && cp -RT ${TEACLAVE_TARGET_DIR}/unix/doc ${TEACLAVE_DOC_INSTALL_DIR}/unix
  DEPENDS prep)

# clippy target
add_custom_target(clippy COMMAND make CLP=1 all)

gen_convenience_targets()
