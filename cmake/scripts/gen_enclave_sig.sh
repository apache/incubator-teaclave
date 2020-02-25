#!/bin/bash
set -e
REQUIRED_ENVS=("TEACLAVE_OUT_DIR" "TEACLAVE_AUDITORS_DIR" "TEACLAVE_EXAMPLE_AUDITORS_DIR" "TEACLAVE_SERVICE_INSTALL_DIR" "TEACLAVE_EXAMPLE_INSTALL_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

cd ${TEACLAVE_OUT_DIR} && cat *_enclave_info.toml > ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml

AUDITOR_PATHS=$(find ${TEACLAVE_AUDITORS_DIR} -mindepth 1 -maxdepth 1 -type d)
for auditor_path in ${AUDITOR_PATHS}; do
auditor=$(basename ${auditor_path})
openssl dgst -sha256 \
        -sign ${TEACLAVE_AUDITORS_DIR}/${auditor}/${auditor}.private.pem \
        -out ${TEACLAVE_AUDITORS_DIR}/${auditor}/${auditor}.sign.sha256 \
        ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml;
done

cp -RT ${TEACLAVE_AUDITORS_DIR}/ ${TEACLAVE_EXAMPLE_AUDITORS_DIR}/
cp -r ${TEACLAVE_AUDITORS_DIR} ${TEACLAVE_TEST_INSTALL_DIR}/
cp ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml ${TEACLAVE_EXAMPLE_INSTALL_DIR}/
cp ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml ${TEACLAVE_TEST_INSTALL_DIR}/
