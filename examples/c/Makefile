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


all: builtin_echo builtin_ordered_set_intersect

CFLAGS := -I../../sdk/c -Ltarget/debug/ -lteaclave_client_sdk

ifeq ($(DCAP),ON)
	CFLAGS += -DDCAP
endif

target/debug/libteaclave_client_sdk.so:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --manifest-path ../../sdk/rust/Cargo.toml --target-dir target

builtin_echo: builtin_echo.c target/debug/libteaclave_client_sdk.so
	gcc -o $@ builtin_echo.c $(CFLAGS)

builtin_ordered_set_intersect: builtin_ordered_set_intersect.c target/debug/libteaclave_client_sdk.so
	gcc -o $@ builtin_ordered_set_intersect.c $(CFLAGS)

run: builtin_echo builtin_ordered_set_intersect
	LD_LIBRARY_PATH=target/debug  ./builtin_echo
	LD_LIBRARY_PATH=target/debug  ./builtin_ordered_set_intersect

clean:
	@rm -rf target
	@rm builtin_echo
	@rm builtin_ordered_set_intersect
