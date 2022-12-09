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

C_SOURCES :=$(wildcard *.c)
C_OBJECTS :=$(C_SOURCES:.c=.o)
C_EXECUTABLES :=$(C_SOURCES:.c=)
C_RUNNERS :=$(C_SOURCES:.c=_runner)

CFLAGS := -I../../sdk/c -Ltarget/debug/ -lteaclave_client_sdk

all: $(C_EXECUTABLES)

ifeq ($(DCAP),ON)
	CFLAGS += -DDCAP
endif

target/debug/libteaclave_client_sdk.so:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --manifest-path ../../sdk/rust/Cargo.toml --target-dir target

%.o: %.c
	gcc -c $(CFLAGS) $<

$(C_EXECUTABLES): target/debug/libteaclave_client_sdk.so
	gcc -o $@ $@.c $(CFLAGS)

$(C_RUNNERS): %_runner: %
	LD_LIBRARY_PATH=target/debug ./$*

run: $(C_RUNNERS)

clean:
	@rm -rf target $(C_OBJECTS) $(C_EXECUTABLES)
