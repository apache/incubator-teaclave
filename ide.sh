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

TRUSTED_TOML="cmake/tomls/Cargo.sgx_trusted_lib.toml"
UNTRUSTED_TOML="cmake/tomls/Cargo.sgx_untrusted_app.toml"
TOML_DEST="Cargo.toml"

TRUSTED_LOCK="third_party/crates-sgx/Cargo.lock"
UNTRUSTED_LOCK="third_party/crates-io/Cargo.lock"
LOCK_DEST="Cargo.lock"

TRUSTED_CONFIG="third_party/crates-sgx/config"
UNTRUSTED_CONFIG="third_party/crates-io/config"
CONFIG_DEST=".cargo/config"

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

copy() {
    # $1: TOML
    # $2: LOCK
    # $3: CONFIG
    mkdir ${script_dir}/.cargo
    cp $1 ${script_dir}/${TOML_DEST}
    cp $2 ${script_dir}/${LOCK_DEST}
    cp $3 ${script_dir}/${CONFIG_DEST}
}

clean() {
    # clean the IDE helper files for Rust
    rm ${script_dir}/${TOML_DEST}
    rm ${script_dir}/${LOCK_DEST}
    rm ${script_dir}/${CONFIG_DEST}
    rm -r ${script_dir}/.cargo
}

main() {

    if [ $1 = "trusted" ]; then
        clean
        copy $TRUSTED_TOML $TRUSTED_LOCK $TRUSTED_CONFIG
    elif [ $1 = "untrusted" ]; then
        clean
        copy $UNTRUSTED_TOML $UNTRUSTED_LOCK $UNTRUSTED_CONFIG
    elif [ $1 = "clean" ]; then
        clean
    else
        echo "Usage: ./ide.sh <trusted|untrusted|clean>"
        return 1
    fi

    return 0
}

main $*
