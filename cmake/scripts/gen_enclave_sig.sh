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

set -e
REQUIRED_ENVS=("TEACLAVE_OUT_DIR" "TEACLAVE_AUDITORS_DIR" "TEACLAVE_EXAMPLE_AUDITORS_DIR" "TEACLAVE_SERVICE_INSTALL_DIR" "TEACLAVE_EXAMPLE_INSTALL_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

if ls "${TEACLAVE_OUT_DIR}"/*_enclave_info.toml > /dev/null 2>&1; then
    cat ${TEACLAVE_OUT_DIR}/*_enclave_info.toml > ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml
fi

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
