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

set -eE

if [ -z "${TEACLAVE_PROJECT_ROOT}" ] \
|| [ -z "${SGX_SDK}" ] || [ -z "${SGX_MODE}" ]; then
    echo "Please set TEACLAVE_PROJECT_ROOT, SGX_SDK and SGX_MODE";
    exit -1
fi

source ${SGX_SDK}/environment
if [ "${SGX_MODE}" = "HW" ]; then
	if [ -z ${AS_ALGO} ] || [ -z ${AS_URL} ] || [ -z ${AS_SPID} ] || [ -z ${AS_KEY} ] ; then
        echo "Please set AS_ALGO, AS_URL, AS_SPID and AS_KEY environment variables."
        exit 1;
    fi
fi

echo_title() {
    width=70
    padding="$(printf '%0.1s' ={1..70})"
    padding_width="$(((width-2-${#1})/2))"
    printf '\e[1m\e[96m%*.*s %s %*.*s\n\e[39m\e[0m' 0 "$padding_width" "$padding" "$1" 0 "$padding_width" "$padding"
}

start_storage_server() {
  python3 ${TEACLAVE_PROJECT_ROOT}/tests/scripts/simple_http_server.py 6789 &
}

run_unit_tests() {
  trap cleanup INT TERM ERR
  pushd ${TEACLAVE_TEST_INSTALL_DIR}

  start_storage_server
  echo_title "encalve unit tests"
  ./teaclave_unit_tests

  popd

  cleanup
}

cleanup() {
  # kill all background services
  [[ -z "$(jobs -p)" ]] || kill -s SIGTERM $(jobs -p)
}

run_integration_tests() {
  trap cleanup INT TERM ERR

  pushd ${TEACLAVE_TEST_INSTALL_DIR}
    echo_title "integration tests"
    ./teaclave_integration_tests
  popd

  echo_title "protected_fs_rs tests (untrusted)"
  pushd ${MT_SGXAPP_TOML_DIR}
  cargo test --manifest-path ${TEACLAVE_PROJECT_ROOT}/common/protected_fs_rs/Cargo.toml \
            --target-dir ${TEACLAVE_TARGET_DIR}/untrusted
  popd

  echo_title "file_agent tests (untrusted)"

  pushd ${TEACLAVE_TEST_INSTALL_DIR}
  start_storage_server
  popd

  pushd ${MT_SGXAPP_TOML_DIR}
  cargo test --manifest-path ${TEACLAVE_PROJECT_ROOT}/file_agent/Cargo.toml \
            --target-dir ${TEACLAVE_TARGET_DIR}/untrusted
  popd

  cleanup
}

run_functional_tests() {
  trap cleanup INT TERM ERR

  echo_title "functional tests"
  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_authentication_service &
  ./teaclave_storage_service &
  sleep 3    # wait for authentication and storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  sleep 3    # wait for management service and scheduler_service
  ./teaclave_access_control_service &
  ./teaclave_frontend_service &
  popd
  sleep 3    # wait for other services

  pushd ${TEACLAVE_TEST_INSTALL_DIR}
  # Run function tests for all except execution service
  ./teaclave_functional_tests -t \
    access_control_service \
    authentication_service \
    frontend_service \
    management_service \
    scheduler_service \
    storage_service

  ${TEACLAVE_CLI_INSTALL_DIR}/teaclave_cli encrypt \
           --algorithm aes-gcm-128 \
           --input-file ./fixtures/fusion/input1.txt \
           --key 00000000000000000000000000000001 \
           --iv 123456781234567812345678 \
           --output-file ./fixtures/fusion/input1.enc

  ${TEACLAVE_CLI_INSTALL_DIR}/teaclave_cli encrypt \
           --algorithm aes-gcm-256 \
           --input-file ./fixtures/fusion/input2.txt \
           --key 0000000000000000000000000000000000000000000000000000000000000002 \
           --iv 012345670123456701234567 \
           --output-file ./fixtures/fusion/input2.enc

  start_storage_server

  # Run tests of execution service separately
  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_execution_service &
  popd
  sleep 3    # wait for execution services

  ./teaclave_functional_tests -t execution_service

  ./teaclave_functional_tests -t end_to_end

  # Run script tests
  ./scripts/functional_tests.py -v

  popd

  # kill all background services
  cleanup
}

run_examples() {
  trap cleanup INT TERM ERR

  echo_title "examples"
  mkdir -p /tmp/fusion_data
  pushd ${TEACLAVE_CLI_INSTALL_DIR}
  ./teaclave_cli verify \
                 --enclave-info ../examples/enclave_info.toml \
                 --public-keys $(find ../examples -name "*.public.pem") \
                 --signatures $(find ../examples -name "*.sign.sha256")
  popd
  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_authentication_service &
  ./teaclave_storage_service &
  sleep 3    # wait for authentication and storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  sleep 3    # wait for management service and scheduler_service
  ./teaclave_access_control_service &
  ./teaclave_frontend_service &
  sleep 3    # wait for other services

  start_storage_server

  # Run tests of execution service separately
  ./teaclave_execution_service &
  sleep 3    # wait for execution services
  popd

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  python3 builtin_echo.py
  python3 mesapy_echo.py
  python3 mesapy_logistic_reg.py
  python3 builtin_gbdt_train.py
  python3 builtin_online_decrypt.py
  python3 builtin_private_join_and_compute.py
  python3 builtin_ordered_set_intersect.py
  python3 builtin_rsa_sign.py
  python3 builtin_face_detection.py
  popd

  # kill all background services
  cleanup
}

case "$1" in
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "functional")
        run_functional_tests
        ;;
    "example")
        run_examples
        ;;
    *)
        run_unit_tests
        run_integration_tests
        run_functional_tests
        run_examples
        ;;
esac
