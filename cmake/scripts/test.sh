#!/bin/bash
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

run_unit_tests() {
  pushd ${TEACLAVE_TEST_INSTALL_DIR}

  echo_title "encalve unit tests"
  ./teaclave_unit_tests

  popd
}

run_integration_tests() {
  pushd ${TEACLAVE_TEST_INSTALL_DIR}

  echo_title "integration tests"
  ./teaclave_integration_tests

  echo_title "protected_fs_rs tests (untrusted)"
  cargo test --manifest-path ${TEACLAVE_PROJECT_ROOT}/common/protected_fs_rs/Cargo.toml \
        --target-dir ${TEACLAVE_TARGET_DIR}/untrusted

  popd
}

run_functional_tests() {
  cleanup() {
        [[ -z "$(jobs -p)" ]] || kill -s SIGTERM $(jobs -p)
  }
  trap cleanup ERR

  pushd ${TEACLAVE_TEST_INSTALL_DIR}

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
  ./teaclave_execution_service &
  popd
  sleep 3    # wait for other services
  ./teaclave_functional_tests

  ./scripts/functional_tests.py -v

  popd

  # kill all background services
  [[ -z "$(jobs -p)" ]] || kill -s SIGTERM $(jobs -p)
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
    *)
        run_unit_tests
        run_integration_tests
        run_functional_tests
        ;;
esac
