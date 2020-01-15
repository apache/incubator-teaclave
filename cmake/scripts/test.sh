#!/bin/bash
set -e
if [ -z "${MESATEE_PROJECT_ROOT}" ] \
|| [ -z "${SGX_SDK}" ] || [ -z "${SGX_MODE}" ]; then
    echo "Please set MESATEE_PROJECT_ROOT, SGX_SDK and SGX_MODE";
    exit -1
fi

source ${SGX_SDK}/environment
if [ "${SGX_MODE}" = "HW" ]; then
	if [ -z ${IAS_SPID} ] || [ -z ${IAS_KEY} ] ; then
        echo "Please set IAS_SPID and IAS_KEY environment variables."
        exit 1;
    fi
fi

echo_title() {
    width=70
    padding="$(printf '%0.1s' ={1..70})"
    padding_width="$(((width-2-${#1})/2))"
    printf '\e[1m\e[96m%*.*s %s %*.*s\n\e[39m\e[0m' 0 "$padding_width" "$padding" "$1" 0 "$padding_width" "$padding"
}

pushd ${MESATEE_TEST_INSTALL_DIR}

echo_title "encalve unit tests"
./teaclave_enclave_unit_test

echo_title "leveldb tests"
./teaclave_leveldb_tests

echo_title "protect_fs tests"
./teaclave_protected_fs_tests
cargo test --manifest-path ${MESATEE_PROJECT_ROOT}/common/protected_fs_rs/Cargo.toml \
      --target-dir ${MESATEE_TARGET_DIR}/untrusted

popd
