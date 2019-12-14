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

cd ${MESATEE_PROJECT_ROOT}/tests && ./module_test.sh
cd ${MESATEE_PROJECT_ROOT}/tests && ./functional_test.sh
cd ${MESATEE_PROJECT_ROOT}/tests && ./integration_test.sh
