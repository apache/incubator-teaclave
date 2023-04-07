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

REQUIRED_ENVS=("TEACLAVE_BIN_INSTALL_DIR" "TEACLAVE_SERVICE_INSTALL_DIR" 
"TEACLAVE_OUT_DIR" "MT_SCRIPT_DIR")

for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

function generate_yaml() {
echo "includes:
  - base.yaml
targets:
  - target: /bin
    copy:
      - files:
        - ${TEACLAVE_BIN_INSTALL_DIR}/teaclave_execution_service_libos
  - target: /opt/occlum/glibc/lib
    copy:
      - files:
        - /opt/occlum/glibc/lib/libnss_dns.so.2
        - /opt/occlum/glibc/lib/libnss_files.so.2
        - /opt/occlum/glibc/lib/libresolv.so.2
        - /lib/x86_64-linux-gnu/libssl.so.1.1
        - /lib/x86_64-linux-gnu/libcrypto.so.1.1
        - /opt/occlum/glibc/lib/librt.so.1
  - target: /etc
    copy:
      - files:
        - /etc/nsswitch.conf
  - target: /
    copy:
      - files:
        - ${TEACLAVE_SERVICE_INSTALL_DIR}/enclave_info.toml
        - ${TEACLAVE_SERVICE_INSTALL_DIR}/runtime.config.toml
      - dirs:
        - ${TEACLAVE_SERVICE_INSTALL_DIR}/auditors      
        
"  > $TEACLAVE_BIN_INSTALL_DIR/teaclave.yaml
}

cd ${TEACLAVE_BIN_INSTALL_DIR}
rm -rf teaclave_instance
occlum new teaclave_instance && cd teaclave_instance && rm -rf image

new_json="$(jq '.resource_limits.user_space_size = "2GB" |
              .resource_limits.kernel_space_heap_size = "320MB" |
              .resource_limits.max_num_of_threads = 700 |
              .resource_limits.kernel_space_stack_size = "10MB" |
              .process.default_heap_size ="256MB" |
              .process.default_mmap_size = "1GB" |
              .env.untrusted += ["TEACLAVE_LOG"] ' Occlum.json)" && \
echo "${new_json}" > Occlum.json
awk '/hostfs/{for(x=NR-2;x<=NR+2;x++)d[x];}{a[NR]=$0}END{for(i=1;i<=NR;i++)if(!(i in d))print a[i]}' Occlum.json > Occlum.json.tmp 
mv Occlum.json.tmp Occlum.json

generate_yaml
copy_bom -f ${TEACLAVE_BIN_INSTALL_DIR}/teaclave.yaml --root image --include-dir /opt/occlum/etc/template
# Required by services
mkdir -p image/tmp/fusion_data
occlum build -f
