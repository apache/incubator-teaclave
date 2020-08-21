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
REQUIRED_ENVS=("TEACLAVE_PROJECT_ROOT" "TEACLAVE_OUT_DIR" "TEACLAVE_TARGET_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

LCOV=lcov
LCOVOPT="--gcov-tool ${TEACLAVE_PROJECT_ROOT}/cmake/scripts/llvm-gcov.sh"
LCOV_REALPATH="${TEACLAVE_PROJECT_ROOT}/cmake/scripts/lcov_realpath.py"
GENHTML=genhtml

cd ${TEACLAVE_PROJECT_ROOT}
find . \( -name "*.gcda" -and \( ! -name "teaclave*" \
     -and ! -name "sgx_cov*" \
     -and ! -name "rusty_leveldb*" \
     -and ! -name "sgx_tprotected_fs*" \
     -and ! -name "protected_fs*" \) \) -exec rm {} \;
cd ${TEACLAVE_PROJECT_ROOT} && \
    for tag in `find ${TEACLAVE_PROJECT_ROOT} -name sgx_cov*.gcda | cut -d'.' -f2`; \
    do mkdir -p ${TEACLAVE_OUT_DIR}/cov_$tag && \
    find ${TEACLAVE_TARGET_DIR} -name *$tag* -exec cp {} ${TEACLAVE_OUT_DIR}/cov_$tag/ \; ; \
    ${LCOV} ${LCOVOPT} --capture \
    --directory ${TEACLAVE_OUT_DIR}/cov_$tag/ --base-directory . \
    -o ${TEACLAVE_OUT_DIR}/modules_$tag.info; done 2>/dev/null
rm -rf ${TEACLAVE_OUT_DIR}/cov_*
cd ${TEACLAVE_PROJECT_ROOT} && ${LCOV} ${LCOVOPT} --capture \
    --directory . --base-directory . \
    -o ${TEACLAVE_OUT_DIR}/modules.info 2>/dev/null
cd ${TEACLAVE_OUT_DIR} && ${LCOV} ${LCOVOPT} $(for tag in \
    `find ${TEACLAVE_PROJECT_ROOT} -name sgx_cov*.gcda | cut -d'.' -f2`; \
    do echo "--add modules_$tag.info"; done) \
    --add modules.info -o merged.info
cd ${TEACLAVE_OUT_DIR} && python ${LCOV_REALPATH} merged.info > merged_realpath.info
${LCOV} ${LCOVOPT} --extract ${TEACLAVE_OUT_DIR}/merged_realpath.info \
    `find ${TEACLAVE_PROJECT_ROOT} -path ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o \
    -path ${TEACLAVE_PROJECT_ROOT}/build -prune -o \
    -path ${TEACLAVE_PROJECT_ROOT}/tests -prune -o \
    -name "*.rs"` -o cov.info
${GENHTML} --branch-coverage --demangle-cpp --legend cov.info \
    -o cov_report --ignore-errors source
