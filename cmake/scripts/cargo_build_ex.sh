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

# extended cargo build script

# if MT_RUSTC_WRAPPER is not empty, use it as rustc
RUSTC="${MT_RUSTC_WRAPPER:-${RUSTC}}"

if [ ! -z "${MUTE_CARGO}" ]; then
    RUSTC="${RUSTC}" cargo build "$@" >/dev/null 2>&1
else
    RUSTC="${RUSTC}" cargo build "$@"
fi

# if CLP is set, run cargo clippy after cargo build
# cannot use MT_RUSTC_WRAPPER for cargo clippy
if [ ! -z "$CLP" ]; then
    cargo clippy "$@" -- -D warnings
fi

# if DOC is set, run cargo doc after cargo build
if [ ! -z "$DOC" ]; then
    RUSTDOCFLAGS="--enable-index-page -Zunstable-options" RUSTC="${RUSTC}" cargo doc "$@"
fi
