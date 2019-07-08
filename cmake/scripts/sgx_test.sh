#!/bin/bash
set -e
if [ -z "${MESATEE_PROJECT_ROOT}" ] || [ -z "${MESATEE_BIN_DIR}" ] \
|| [ -z "${SGX_SDK}" ] || [ -z "${SGX_MODE}" ]; then
    echo "Please set MESATEE_PROJECT_ROOT, MESATEE_BIN_DIR, SGX_SDK and SGX_MODE";
    exit -1
fi

source ${SGX_SDK}/environment
if [ "${SGX_MODE}" = "HW" ]; then
	if [ ! -f ${MESATEE_BIN_DIR}/ias_spid.txt ] || [ ! -f ${MESATEE_BIN_DIR}/ias_key.txt ] ; then
        echo "Please follow \"How to Run (SGX)\" in README to obtain \
ias_spid.txt and ias_key.txt, and put in the bin";
        exit 1;
    fi
fi
cd ${MESATEE_PROJECT_ROOT}/tests && ./functional_test.sh
cd ${MESATEE_PROJECT_ROOT}/tests && ./integration_test.sh
