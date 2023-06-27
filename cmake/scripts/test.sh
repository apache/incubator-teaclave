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
export TEACLAVE_LOG=teaclave=info

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
  wait_port 6789
}

run_unit_tests() {
  trap cleanup INT TERM ERR

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python/wasm_c_millionaire_problem_payload
  make
  popd

  pushd ${TEACLAVE_TEST_INSTALL_DIR}

  start_storage_server
  echo_title "encalve unit tests"
  rm -rf mock_db_unit_test
  ./teaclave_unit_tests

  popd

  cleanup
}

cleanup() {
  # gracefully terminate all background services with SIGTERM
  [[ -z "$(jobs -p -r)" ]] || kill -s SIGTERM $(jobs -p -r)
  wait # wait for resource release
  echo "All jobs terminated."
}

wait_port() {
  for port in "$@"
  do
    timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' localhost "$port"
  done
}

generate_python_grpc_stubs() {
  python3 -m grpc_tools.protoc \
          --proto_path=${TEACLAVE_PROJECT_ROOT}/services/proto/src/proto \
          --python_out=${TEACLAVE_PROJECT_ROOT}/sdk/python \
          --grpclib_python_out=${TEACLAVE_PROJECT_ROOT}/sdk/python \
          ${TEACLAVE_PROJECT_ROOT}/services/proto/src/proto/*.proto
}

run_integration_tests() {
  trap cleanup INT TERM ERR

  pushd ${TEACLAVE_TEST_INSTALL_DIR}
    echo_title "integration tests"
    ./teaclave_integration_tests
  popd

  echo_title "file_agent tests (untrusted)"

  pushd ${TEACLAVE_TEST_INSTALL_DIR}
  start_storage_server
  popd

  pushd ${MT_SGXAPP_TOML_DIR}
  cargo test -p teaclave_file_agent --target-dir ${TEACLAVE_TARGET_DIR}/untrusted
  popd

  cleanup
}

run_functional_tests() {
  trap cleanup INT TERM ERR

  echo_title "functional tests"
  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_authentication_service &
  ./teaclave_storage_service &
  ./teaclave_access_control_service &
  wait_port 7776 17776 17778 17779 # wait for access control, authentication, storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  wait_port 17777 17780 # wait for management service and scheduler_service
  ./teaclave_frontend_service &
  wait_port 7777 # wait for other services
  popd

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

  generate_python_grpc_stubs
  
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  ./scripts/functional_tests.py -v

  popd

  # kill all background services
  cleanup
}

run_sdk_tests() {
  trap cleanup INT TERM ERR

  echo_title "SDK tests"
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
  ./teaclave_access_control_service &
  wait_port 7776 17776 17778 17779 # wait for access control, authentication, storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  wait_port 17777 17780 # wait for management service and scheduler_service
  ./teaclave_frontend_service &
  wait_port 7777 # wait for other services

  start_storage_server

  # Run tests of execution service separately
  ./teaclave_execution_service &
  sleep 3    # wait for execution services
  popd

  pushd ${MT_SGXAPP_TOML_DIR}
  RUSTFLAGS=${RUSTFLAGS} cargo test --manifest-path ${TEACLAVE_PROJECT_ROOT}/sdk/rust/Cargo.toml \
        --target-dir ${TEACLAVE_TARGET_DIR}/untrusted
  popd

  # kill all background services
  cleanup
}

mesapy_examples() {
  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  python3 mesapy_echo.py
  python3 mesapy_logistic_reg.py
  python3 mesapy_optional_files.py
  popd
}

builtin_examples() {
  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  python3 builtin_echo.py
  python3 builtin_gbdt_train.py
  python3 builtin_online_decrypt.py
  python3 builtin_private_join_and_compute.py
  python3 builtin_ordered_set_intersect.py
  python3 builtin_rsa_sign.py
  python3 builtin_face_detection.py
  python3 builtin_password_check.py
  python3 test_disable_function.py
  popd

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/c
  make run
  popd

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/rust
  pushd ./builtin_echo
  RUSTFLAGS=${RUSTFLAGS} cargo run
  popd
  pushd ./builtin_ordered_set_intersect
  RUSTFLAGS=${RUSTFLAGS} cargo run
  popd
  pushd ./sequential_functions
  RUSTFLAGS=${RUSTFLAGS} cargo run
  popd
  popd
}

wasm_examples() {
  # Generate WASM file for WAMR Rust example
  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python/wasm_c_simple_add_payload
  make
  popd

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python/wasm_rust_psi_payload
  make
  popd

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  python3 wasm_c_simple_add.py
  python3 wasm_rust_psi.py
  popd
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
  ./teaclave_access_control_service &
  wait_port 7776 17776 17778 17779 # wait for access control, authentication, storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  wait_port 17777 17780 # wait for management service and scheduler_service
  ./teaclave_frontend_service &
  wait_port 7777 # wait for other services

  start_storage_server

  # Run tests of execution service separately
  ./teaclave_execution_service &
  sleep 3    # wait for execution services
  popd

  generate_python_grpc_stubs

  # run builtin examples
  builtin_examples

  # run wasm examples
  wasm_examples

  # kill all background services
  cleanup
}

run_libos_examples() {
  trap cleanup INT TERM ERR

  echo_title "libos examples"
  if [ "${SGX_MODE}" = "HW" ]; then
      echo "Executing LibOS's examples in SGX HW mode is not currently supported."
      exit
  fi
  mkdir -p /tmp/fusion_data

  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_authentication_service &
  ./teaclave_storage_service &
  ./teaclave_access_control_service &
  wait_port 7776 17776 17778 17779 # wait for access control, authentication, storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  wait_port 17777 17780 # wait for management service and scheduler_service
  ./teaclave_frontend_service &
  wait_port 7777 # wait for other services

  start_storage_server

  pushd ${TEACLAVE_BIN_INSTALL_DIR}
  cp -rf ${TEACLAVE_SERVICE_INSTALL_DIR}/auditors .
  cp -f ${TEACLAVE_SERVICE_INSTALL_DIR}/runtime.config.toml .
  cp -f ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml .
  # Run tests of libos service separately
  ./teaclave_execution_service_libos &
  sleep 3    # wait for execution services
  popd

  generate_python_grpc_stubs
          
  # run builtin examples
  builtin_examples

  # run wasm examples
  wasm_examples
  
  # kill all background services
  cleanup
}

run_cancel_test() {
  trap cleanup INT TERM ERR

  echo_title "cancel"
  mkdir -p /tmp/fusion_data
  pushd ${TEACLAVE_CLI_INSTALL_DIR}
  ./teaclave_cli verify \
                 --enclave-info ../examples/enclave_info.toml \
                 --public-keys $(find ../examples -name "*.public.pem") \
                 --signatures $(find ../examples -name "*.sign.sha256")
  popd

  echo "initiating Teaclave with 2 executors..."

  pushd ${TEACLAVE_SERVICE_INSTALL_DIR}
  ./teaclave_authentication_service &
  ./teaclave_storage_service &
  ./teaclave_access_control_service &
  wait_port 7776 17776 17778 17779 # wait for access control, authentication, storage service
  ./teaclave_management_service &
  ./teaclave_scheduler_service &
  wait_port 17777 17780 # wait for management service and scheduler_service
  ./teaclave_frontend_service &
  wait_port 7777 # wait for other services

  start_storage_server

  # Run of execution services separately
  ./teaclave_execution_service & exe_pid1=$!
  ./teaclave_execution_service & exe_pid2=$!
  sleep 10    # wait for execution services
  popd

  echo "executor 1 pid: $exe_pid1"
  echo "executor 2 pid: $exe_pid2"

  generate_python_grpc_stubs

  pushd ${TEACLAVE_PROJECT_ROOT}/examples/python
  export PYTHONPATH=${TEACLAVE_PROJECT_ROOT}/sdk/python
  python3 mesapy_deadloop_cancel.py
  popd

  sleep 10 # Wait for executor exit

  live_pids=0
  if ps -p $exe_pid1 > /dev/null
  then
    live_pids=$((live_pids+1))
  fi

  if ps -p $exe_pid2 > /dev/null
  then
    live_pids=$((live_pids+1))
  fi

  if [ $live_pids -eq 1 ]
  then
    echo "only one executor is killed, test passed"
  else
    echo "Some unexpected happens, test failed"
    false
  fi

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
    "sdk")
        run_sdk_tests
        ;;
    "example")
        run_examples
        ;;
    "cancel")
        run_cancel_test
        ;;
    "libos")
        run_libos_examples
        ;;
    *)
        run_unit_tests
        run_integration_tests
        run_functional_tests
        run_sdk_tests
        run_examples
        run_libos_examples
        ;;
esac
