#!/bin/bash
set -e
if [ -z "${MESATEE_PROJECT_ROOT}" ] || [ -z "${MESATEE_BIN_DIR}" ] \
|| [ -z "${SGX_SDK}" ] || [ -z "${SGX_MODE}" ]; then
    echo "Please set MESATEE_PROJECT_ROOT, MESATEE_BIN_DIR, SGX_SDK and SGX_MODE";
    exit -1
fi

source ${SGX_SDK}/environment
if [ "${SGX_MODE}" = "HW" ]; then
	if [ -z ${IAS_SPID} ] || [ -z ${IAS_KEY} ] ; then
        echo "SGX launch check failed: Env var for IAS SPID or IAS KEY does NOT exist. Please follow \"How to Run (SGX)\" in README to obtain, and specify the value in environment variables and put the names of environment variables in config.toml. The default variables are IAS_SPID and IAS_KEY."
        exit 1;
    fi
fi

cd ${MESATEE_PROJECT_ROOT}/tests && ./module_test.sh
cd ${MESATEE_PROJECT_ROOT}/tests && ./functional_test.sh
cd ${MESATEE_PROJECT_ROOT}/tests && ./integration_test.sh
