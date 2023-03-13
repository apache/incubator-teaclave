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

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
teaclave_root=${script_dir}/..

trusted_toml="${teaclave_root}/cmake/tomls/Cargo.sgx_trusted_lib.toml"
untrusted_toml="${teaclave_root}/cmake/tomls/Cargo.sgx_untrusted_app.toml"
toml_dest="Cargo.toml"

trusted_lock="${teaclave_root}/third_party/crates-sgx/Cargo.lock"
untrusted_lock="${teaclave_root}/third_party/crates-io/Cargo.lock"
lock_dest="Cargo.lock"

trusted_config="${teaclave_root}/third_party/crates-sgx/config"
untrusted_config="${teaclave_root}/third_party/crates-io/config"
config_dest=".cargo/config"


copy() {
    toml="$1"
    lock="$2"
    config="$3"
    mkdir "${teaclave_root}"/.cargo
    cp "${toml}" "${teaclave_root}"/${toml_dest}
    cp "${lock}" "${teaclave_root}"/${lock_dest}
    cp "${config}" "${teaclave_root}"/${config_dest}
}

clean() {
    # clean the IDE helper files for Rust
    rm "${teaclave_root}"/${toml_dest}
    rm "${teaclave_root}"/${lock_dest}
    rm "${teaclave_root}"/${config_dest}
    rm -r "${teaclave_root}"/.cargo
}

main() {

    if [ "$1" = "trusted" ]; then
        clean 2>/dev/null
        copy "$trusted_toml" "$trusted_lock" "$trusted_config"
        sed -i '/directory = "vendor"/c\directory = "third_party/crates-sgx/vendor"' "${teaclave_root}"/${config_dest}
    elif [ "$1" = "untrusted" ]; then
        clean 2>/dev/null
        copy "$untrusted_toml" "$untrusted_lock" "$untrusted_config"
        sed -i '/directory = "vendor"/c\directory = "third_party/crates-io/vendor"' "${teaclave_root}"/${config_dest}
    elif [ "$1" = "clean" ]; then
        clean
    else
        echo "Usage: ./ide.sh <trusted|untrusted|clean>"
        return 1
    fi

    return 0
}

main "$*"
