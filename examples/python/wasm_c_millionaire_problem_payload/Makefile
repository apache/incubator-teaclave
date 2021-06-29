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

TARGET = millionaire_problem.wasm

# Set CC to your clang with WASM target support or which shipped with wasi-sdk
# This example uses wasi-sdk's clang
CC = clang
# Notice: if you're using original clang, you may encounter an error that clang
# cannot find `wasm-ld`. You may create a soft link `wasm-ld` pointing to
# `wasm-ld-x`(version number) 

CFLAGS = -nostdlib
# set target if you're using original clang
# CFLAGS += --target=wasm32

CC_LINKER_FLAGS = -Wl,--export-all \
	-Wl,--allow-undefined \
	-Wl,--no-entry

INCLUDES := -I../../../sdk/payload/wasm/

CFLAGS += $(CC_LINKER_FLAGS) $(INCLUDES)

all: $(TARGET)

$(TARGET): millionaire_problem.c
	$(CC) $(CFLAGS) -o $@ $<

clean:
	@rm ./*.wasm
