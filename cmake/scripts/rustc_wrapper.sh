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
REQUIRED_ENVS=("TEACLAVE_PROJECT_ROOT" "TEACLAVE_BUILD_ROOT" "TEACLAVE_SYMLINKS")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

# Tell rustc to remap absolute src paths to make enclaves' signature more reproducible
exec rustc "$@" --remap-path-prefix=${HOME}/.cargo=${TEACLAVE_SYMLINKS}/cargo_home --remap-path-prefix=${TEACLAVE_PROJECT_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_src --remap-path-prefix=${TEACLAVE_BUILD_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_build
