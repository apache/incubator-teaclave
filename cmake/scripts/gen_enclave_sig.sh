#!/bin/bash
set -e
REQUIRED_ENVS=("MESATEE_OUT_DIR" "MESATEE_AUDITORS_DIR" "MESATEE_SERVICE_INSTALL_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

cd ${MESATEE_OUT_DIR} && cat *_enclave_info.txt > ${MESATEE_SERVICE_INSTALL_DIR}/enclave_info.txt

AUDITOR_PATHS=$(find ${MESATEE_AUDITORS_DIR} -mindepth 1 -maxdepth 1 -type d)
for auditor_path in ${AUDITOR_PATHS}; do
auditor=$(basename ${auditor_path})
openssl dgst -sha256 \
        -sign ${MESATEE_AUDITORS_DIR}/${auditor}/${auditor}.private.pem \
        -out ${MESATEE_AUDITORS_DIR}/${auditor}/${auditor}.sign.sha256 \
        ${MESATEE_SERVICE_INSTALL_DIR}/enclave_info.txt;
done
