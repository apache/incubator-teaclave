#!/bin/bash
if [ -z "$MESATEE_OUT_DIR" ] || [ -z "${MESATEE_AUDITORS_DIR}" ]; then
    echo "Please set MESATEE_OUT_DIR and MESATEE_AUDITORS_DIR";
    exit -1
fi

cd ${MESATEE_OUT_DIR} && cat *_enclave_info.txt > enclave_info.txt

AUDITOR_PATHS=$(find ${MESATEE_AUDITORS_DIR} -mindepth 1 -maxdepth 1 -type d)
for auditor_path in ${AUDITOR_PATHS}; do
auditor=$(basename ${auditor_path})
openssl dgst -sha256 \
        -sign ${MESATEE_AUDITORS_DIR}/${auditor}/${auditor}.private.pem \
        -out ${MESATEE_AUDITORS_DIR}/${auditor}/${auditor}.sign.sha256 \
        ${MESATEE_PROJECT_ROOT}/out/enclave_info.txt;
done
