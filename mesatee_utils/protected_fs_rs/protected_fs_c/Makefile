#
# Copyright (C) 2011-2019 Intel Corporation. All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
#   * Redistributions of source code must retain the above copyright
#     notice, this list of conditions and the following disclaimer.
#   * Redistributions in binary form must reproduce the above copyright
#     notice, this list of conditions and the following disclaimer in
#     the documentation and/or other materials provided with the
#     distribution.
#   * Neither the name of Intel Corporation nor the names of its
#     contributors may be used to endorse or promote products derived
#     from this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#
#

PROTECTED_FS_PROJECT_ROOT ?= $(CURDIR)
PROTECTED_FS_OUT_DIR ?= /tmp/protected_fs
PROTECTED_FS_LIB_NAME ?= libprotected_fs.a
TSGX_DIR := sgx_tprotected_fs
USGX_DIR := sgx_uprotected_fs

PROJECT_ROOT := $(PROTECTED_FS_PROJECT_ROOT)
OUT_DIR := $(PROTECTED_FS_OUT_DIR)

INCLUDE += -I$(PROJECT_ROOT) -I$(PROJECT_ROOT)/inc
CXXFLAGS += -Werror -U__STRICT_ANSI__ -std=c++11 -lpthread
CXXFLAGS += -Wno-unused-local-typedefs -Wno-shadow -Wno-missing-field-initializers -Wno-unused-parameter


ABS_SRC := $(wildcard $(PROJECT_ROOT)/$(TSGX_DIR)/*.cpp) $(wildcard $(PROJECT_ROOT)/$(USGX_DIR)/*.cpp)
SORTED_ABS_SRC := $(sort $(ABS_SRC))
SORTED_ABS_OBJ := $(SORTED_ABS_SRC:.cpp=.o)

ABS_OBJ := $(patsubst $(PROTECTED_FS_PROJECT_ROOT)/%,$(OUT_DIR)/%,$(SORTED_ABS_OBJ))
SRC := $(ABS_SRC)
OBJ := $(ABS_OBJ)

EXAMPLE_DIR := $(PROTECTED_FS_PROJECT_ROOT)/example
EXAMPLE_SRC := $(EXAMPLE_DIR)/example.c
EXAMPLE_EXE := $(EXAMPLE_DIR)/example
EXAMPLE_OUTPUT := $(EXAMPLE_DIR)/data_file
EXAMPLE_DEP_OPTION := -lcrypto

TARGET:= $(OUT_DIR)/$(PROTECTED_FS_LIB_NAME)

.PHONY: all
all: $(TARGET)

.PHONY: example
example: $(EXAMPLE_EXE)

$(TARGET): $(OBJ)
	$(AR) rcsD $@ $(OBJ)
	
$(OUT_DIR)/%.o: $(PROTECTED_FS_PROJECT_ROOT)/%.cpp
	mkdir -p $(OUT_DIR)/$(TSGX_DIR)
	mkdir -p $(OUT_DIR)/$(USGX_DIR)
	$(CXX) $(CXXFLAGS) $(INCLUDE)  -c $< -o $@

$(EXAMPLE_EXE): $(TARGET)
	$(CXX) -m64 -O2 $(INCLUDE) $(EXAMPLE_SRC) $(TARGET) $(EXAMPLE_DEP_OPTION) -o $(EXAMPLE_EXE) 

.PHONY: clean
clean:
	@$(RM) $(OBJ)
	@$(RM) $(TARGET) 
	@$(RM) $(EXAMPLE_EXE) 
	@$(RM) $(EXAMPLE_OUTPUT) 

.PHONY: rebuild
rebuild: 
	$(MAKE) clean 
	$(MAKE) all