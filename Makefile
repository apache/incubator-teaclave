# Copyright 2019 MesaTEE Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

SHELL = /bin/bash

MESAPY_VERSION = da84c8c65d400581a7c17aab06751eace42ef90a

SGX_ENCLAVE_FEATURES = -Z package-features --features mesalock_sgx
ifeq ($(DBG),) 	# Release build
	TARGET = release
	CARGO_BUILD_FLAGS = --release
else			# Debug build
	TARGET = debug
ifneq ($(COV),)	# Debug build + coverage collection
	SGX_ENCLAVE_FEATURES = -Z package-features --features "mesalock_sgx cov"
	COV_FLAGS = CARGO_INCREMENTAL=0 \
		RUSTFLAGS="-D warnings -Zprofile -Ccodegen-units=1 \
		-Cllvm_args=-inline-threshold=0 \
		-Coverflow-checks=off -Zno-landing-pads"
endif
endif

MESATEE_PROJECT_ROOT ?= $(CURDIR)
RUST_SGX_SDK := $(MESATEE_PROJECT_ROOT)/third_party/rust-sgx-sdk
MESATEE_CFG_DIR ?= $(MESATEE_PROJECT_ROOT)
MESATEE_BUILD_CFG_DIR ?= $(MESATEE_PROJECT_ROOT)
SGX_SDK ?= /opt/sgxsdk
SGX_MODE ?= HW

SGX_EDGER8R := $(SGX_SDK)/bin/x64/sgx_edger8r
SGX_ENCLAVE_SIGNER := $(SGX_SDK)/bin/x64/sgx_sign
SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
SGX_COMMON_CFLAGS := -m64 -O2
SGX_UNTRUSTED_CFLAGS := $(SGX_COMMON_CFLAGS) -fPIC -Wno-attributes \
	-I$(SGX_SDK)/include -I$(RUST_SGX_SDK)/edl
SGX_TRUSTED_CFLAGS := $(SGX_COMMON_CFLAGS) -nostdinc -fvisibility=hidden \
	-fpie -fstack-protector \
	-I$(RUST_SGX_SDK)/edl -I$(RUST_SGX_SDK)/common/inc \
	-I$(SGX_SDK)/include -I$(SGX_SDK)/include/tlibc \
	-I$(SGX_SDK)/include/stlport -I$(SGX_SDK)/include/epid

ifneq ($(SGX_MODE), HW)
	Trts_Library_Name := sgx_trts_sim
	Service_Library_Name := sgx_tservice_sim
else
	Trts_Library_Name := sgx_trts
	Service_Library_Name := sgx_tservice
endif

MODULES_DIR := $(MESATEE_PROJECT_ROOT)
TOOLCHAIN_DEPS_DIR := $(MESATEE_PROJECT_ROOT)/toolchain_deps
THIRD_PARTY_DIR := $(MESATEE_PROJECT_ROOT)/third_party
OUT_DIR := $(MESATEE_PROJECT_ROOT)/out
MESATEE_BIN_DIR ?= $(MESATEE_PROJECT_ROOT)/bin
MESATEE_AUDITORS_DIR ?= $(MESATEE_PROJECT_ROOT)/auditors
TARGET_DIR = $(MODULES_DIR)/target
UNTRUSTED_TARGET_DIR := $(TARGET_DIR)/untrusted
UNIX_TARGET_DIR := $(TARGET_DIR)/unix
TRUSTED_TARGET_DIR := $(TARGET_DIR)/trusted
TEE_BINDER_DIR := $(MODULES_DIR)/mesatee_binder
EDL_FILE := $(TEE_BINDER_DIR)/Enclave.edl

SGX_MODULES := mesatee_services/kms mesatee_services/tdfs mesatee_services/tms \
	mesatee_services/fns tests/functional_test
SGX_LIBS :=
UNIX_MODULES := integration_test private_join_and_compute quickstart \
	image_resizing online_decrypt rsa_sign py_matrix_multiply py_file kmeans \
	logistic_reg lin_reg svm gen_linear_model gaussian_mixture_model \
	gaussian_processes dbscan neural_net naive_bayes gbdt mesatee_cli
UNIX_LIBS := mesatee_sdk protected_fs_rs
LIBS := $(SGX_LIBS) $(UNIX_LIBS)

LCOV := lcov
LCOVOPT := --gcov-tool $(TOOLCHAIN_DEPS_DIR)/llvm-gcov
GENHTML := genhtml

all: sgx unix

prep: check-sgx-sdk init-submodules
	$(call sgx_build_clean)
	mkdir -p $(MESATEE_BIN_DIR) $(OUT_DIR)
	rustup install --no-self-update $(TOOLCHAIN) > /dev/null 2>&1
	cd $(OUT_DIR) && wget -qN \
		https://mesapy.org/release/$(MESAPY_VERSION)-mesapy-sgx.tar.gz && \
		tar xzf $(MESAPY_VERSION)-mesapy-sgx.tar.gz
	# Tell gcc/clang to remap absolute src paths to make enclaves' signature more reproducible
	echo 'exec cc "$$''@"' " -fdebug-prefix-map=${MESATEE_PROJECT_ROOT}=/mesatee_src" > $(OUT_DIR)/cc_wrapper.sh
	chmod +x $(OUT_DIR)/cc_wrapper.sh
	# Tell rustc to remap absolute src paths to make enclaves' signature more reproducible
	echo 'exec rustc "$$''@"' " --remap-path-prefix=${HOME}/.cargo=/cargo_home --remap-path-prefix=${MESATEE_PROJECT_ROOT}=/mesatee_src" > $(OUT_DIR)/rustc_wrapper.sh
	chmod +x $(OUT_DIR)/rustc_wrapper.sh

check-sgx-sdk:
	if [ ! -d $(SGX_SDK) ] ; then \
		echo "SGX SDK not found at $(SGX_SDK), \
please adjust the SGX_SDK env or the Makefile"; exit 1; fi

init-submodules:
	if git submodule status | egrep -q '^[-]|^[+]' ; then \
		echo "INFO: Need to reinitialize git submodules"; \
		git submodule update --init --recursive; \
	fi

# "=" gurantees lazy evaluation until rust-sgx-sdk submodule is populated
TOOLCHAIN = $(shell cat third_party/rust-sgx-sdk/rust-toolchain)

# arg1: build dir
# arg2: target output dir
# arg3: extra build params
ifndef VERBOSE
define cargo_build
	cd $(1) && \
	RUSTUP_TOOLCHAIN=$(TOOLCHAIN) \
	RUSTC=$(OUT_DIR)/rustc_wrapper.sh \
	CC=$(OUT_DIR)/cc_wrapper.sh \
	$(COV_FLAGS) unbuffer cargo build --target-dir $(2) \
	$(CARGO_BUILD_FLAGS) $(3) 2>&1 | \
	while read l; do if grep -q \
	"Updating\|Downloaded\|Compiling" <<< $$l; \
	then echo -ne '\033[2K'$${l%%(*}'\r'; \
	else echo -e '\033[2K'"$$l"; fi; done; \
	if [ $${PIPESTATUS[0]} -ne 0 ]; then exit 1; fi
endef
else
define cargo_build
	cd $(1) && \
	RUSTUP_TOOLCHAIN=$(TOOLCHAIN) \
	RUSTC=$(OUT_DIR)/rustc_wrapper.sh \
	CC=$(OUT_DIR)/cc_wrapper.sh \
	$(COV_FLAGS) cargo build --target-dir $(2) \
	$(CARGO_BUILD_FLAGS) $(3); \
	if [ $$? -ne 0 ]; then exit 1; fi
endef
endif

define cargo_clean
	cd $(1) && \
	cargo clean
endef

define sgx_build_prepare
	mkdir -p $(MESATEE_PROJECT_ROOT)/.cargo
	$(call cargo_toml_prepare, sgx_trusted_lib)
	cp -f $(THIRD_PARTY_DIR)/crates-sgx/Cargo.lock $(MODULES_DIR)/Cargo.lock
	cp -f $(THIRD_PARTY_DIR)/crates-sgx/config $(MESATEE_PROJECT_ROOT)/.cargo/config
	sed -i 's/directory = "vendor"/directory = "third_party\/crates-sgx\/vendor"/' $(MESATEE_PROJECT_ROOT)/.cargo/config
	rm -f $(OUT_DIR)/enclave_info.txt
endef

sgx_build_prepare:
	$(call sgx_build_prepare)

define sgx_build_clean
	$(call cargo_toml_clean)
	rm -f $(MODULES_DIR)/Cargo.lock
	rm -f $(MESATEE_PROJECT_ROOT)/.cargo/config
endef

define cargo_toml_prepare
	cp -f $(TOOLCHAIN_DEPS_DIR)/Cargo.$(strip $(1)).toml $(MODULES_DIR)/Cargo.toml
endef

define cargo_toml_clean
	rm -f $(MODULES_DIR)/Cargo.toml 
endef

sgx_build_clean:
	$(call sgx_build_clean)


#arg1: module name
#arg2: enclave config
define sgx_link
	cd $(OUT_DIR) && $(CC) libEnclave_t.o ffi.o -o \
		$(OUT_DIR)/$(strip $(1)).enclave.so $(SGX_COMMON_CFLAGS) \
		-Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
		-L$(SGX_LIBRARY_PATH) -Wl,--whole-archive -l$(Trts_Library_Name)  \
		-Wl,--no-whole-archive -Wl,--start-group \
		-l$(Service_Library_Name) -lsgx_tprotected_fs -lsgx_tkey_exchange\
		-lsgx_tstdc -lsgx_tcxx -lsgx_tservice -lsgx_tcrypto \
		-L$(OUT_DIR) -lpypy-c -lsgx_tlibc_ext -lffi \
		-L$(TRUSTED_TARGET_DIR)/$(TARGET) -l$(1)_enclave -Wl,--end-group \
		-Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
		-Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
		-Wl,--defsym,__ImageBase=0 \
		-Wl,--gc-sections \
		-Wl,--version-script=$(TOOLCHAIN_DEPS_DIR)/Enclave.lds && \
	$(SGX_ENCLAVE_SIGNER) sign -key $(TOOLCHAIN_DEPS_DIR)/Enclave_private.pem \
		-enclave $(strip $(1)).enclave.so \
		-out $(MESATEE_BIN_DIR)/$(strip $(1)).enclave.signed.so \
		-config $(MESATEE_PROJECT_ROOT)/$(strip $(2)) \
		-dumpfile $(strip $(1)).enclave.meta.txt > /dev/null 2>&1 && \
	echo $(strip $(1)) >> enclave_info.txt && \
	grep -m1 -A2 "mrsigner->value" $(strip $(1)).enclave.meta.txt >> enclave_info.txt && \
	grep -m1 -A2 "body.enclave_hash" $(strip $(1)).enclave.meta.txt >> enclave_info.txt
endef

BOLD=\033[1;32m
END_BOLD=\033[0m

config_gen: prep
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	$(call cargo_toml_prepare, unix_app)
	$(call cargo_build, $(MODULES_DIR), $(UNIX_TARGET_DIR), -p $@)
	$(call cargo_toml_clean)
	cp $(UNIX_TARGET_DIR)/$(TARGET)/$@ $(MESATEE_BIN_DIR)

sgx_pregen: prep $(EDL_FILE) $(SGX_EDGER8R)
	$(SGX_EDGER8R) --untrusted $(EDL_FILE) --search-path $(SGX_SDK)/include \
		--search-path $(RUST_SGX_SDK)/edl --untrusted-dir $(OUT_DIR)
	cd $(OUT_DIR) && $(CC) $(SGX_UNTRUSTED_CFLAGS) -c Enclave_u.c -o libEnclave_u.o
	cd $(OUT_DIR) && $(AR) rcsD libEnclave_u.a libEnclave_u.o
	$(SGX_EDGER8R) --trusted $(EDL_FILE) --search-path $(SGX_SDK)/include \
		--search-path $(RUST_SGX_SDK)/edl --trusted-dir $(OUT_DIR)
	cd $(OUT_DIR) && $(CC) $(SGX_TRUSTED_CFLAGS) -c Enclave_t.c -o libEnclave_t.o

$(SGX_MODULES): config_gen sgx_pregen
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	$(call cargo_toml_prepare, sgx_untrusted_app)
	$(call cargo_build, $(MODULES_DIR), $(UNTRUSTED_TARGET_DIR), -p $(notdir $@))
	$(call cargo_toml_clean)
	cp $(UNTRUSTED_TARGET_DIR)/$(TARGET)/$(notdir $@) $(MESATEE_BIN_DIR)

	echo -e "$(BOLD)[*] Building $@_enclave$(END_BOLD)"
	$(call sgx_build_prepare)
	$(call cargo_build, $(MODULES_DIR), $(TRUSTED_TARGET_DIR), $(SGX_ENCLAVE_FEATURES) -p $(notdir $@)_enclave)
	$(call sgx_build_clean)
	$(call sgx_link, $(notdir $@), $@/sgx_trusted_lib/Enclave.config.xml)

sgx_untrusted: config_gen sgx_pregen
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	$(call cargo_toml_prepare, sgx_untrusted_app)
	$(call cargo_build, $(MODULES_DIR), $(UNTRUSTED_TARGET_DIR), )
	$(call cargo_toml_clean)
	for m in $(SGX_MODULES); do cp $(UNTRUSTED_TARGET_DIR)/$(TARGET)/$${m##*/} $(MESATEE_BIN_DIR); done

sgx_trusted: config_gen sgx_pregen sgx_untrusted
	$(call sgx_build_prepare)
	for m in $(SGX_MODULES); do \
		echo -e "$(BOLD)[*] Building $${m}_enclave$(END_BOLD)" && \
		$(call cargo_build, $(MODULES_DIR), $(TRUSTED_TARGET_DIR), \
			$(SGX_ENCLAVE_FEATURES) -p $${m##*/}_enclave) && \
		$(call sgx_link, $${m##*/}, $$m/sgx_trusted_lib/Enclave.config.xml); done && \
	set -e && for auditor in $(shell ls $$MESATEE_AUDITORS_DIR -I "*.md"); do \
		openssl dgst -sha256 \
			-sign $(MESATEE_AUDITORS_DIR)/$${auditor}/$${auditor}.private.pem \
			-out $(MESATEE_AUDITORS_DIR)/$${auditor}/$${auditor}.sign.sha256 \
			$(MESATEE_PROJECT_ROOT)/out/enclave_info.txt; done
	$(call sgx_build_clean)

$(UNIX_MODULES): prep config_gen
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	$(call cargo_toml_prepare, unix_app)
	$(call cargo_build, $(MODULES_DIR), $(UNIX_TARGET_DIR), -p $@)
	$(call cargo_toml_clean)
	cp $(UNIX_TARGET_DIR)/$(TARGET)/$@ $(MESATEE_BIN_DIR)

$(UNIX_LIBS): prep config_gen
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	cp -f $(TOOLCHAIN_DEPS_DIR)/Cargo.unix_app.toml $(MODULES_DIR)/Cargo.toml
	$(call cargo_build, $(MODULES_DIR), $(UNIX_TARGET_DIR), -p $@)

mesatee_sdk_c: prep mesatee_sdk
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	$(call cargo_toml_prepare, unix_app)
	$(call cargo_build, $(MODULES_DIR), $(UNIX_TARGET_DIR), -p $@)
	$(call cargo_toml_clean)
	cp $(UNIX_TARGET_DIR)/$(TARGET)/*.so $(MESATEE_BIN_DIR)
	cp -r $(MODULES_DIR)/mesatee_sdk/c_sdk/include/mesatee $(MESATEE_BIN_DIR)

quickstart_c: mesatee_sdk_c
	echo -e "$(BOLD)[*] Building $@$(END_BOLD)"
	gcc -o $(MESATEE_BIN_DIR)/$@ \
		$(MODULES_DIR)/examples/quickstart_c/main.c \
		-I$(MESATEE_BIN_DIR)/ \
		-L$(MESATEE_BIN_DIR)/ \
		-lmesatee_sdk_c \
		-Wl,-rpath $(MESATEE_BIN_DIR)

functional_test: tests/functional_test
fns: mesatee_services/fns
kms: mesatee_services/kms
tms: mesatee_services/tms
tdfs: mesatee_services/tdfs

sgx: sgx_trusted sgx_untrusted

unix: sgx $(UNIX_MODULES) mesatee_sdk_c quickstart_c

examples: private_join_and_compute image_resizing online_decrypt quickstart_c

cov:
	find . \( -name "*.gcda" -and \( ! -name "sgx_cov*" \
		-and ! -name "kms*" -and ! -name "fns*" \
		-and ! -name "tdfs*" -and ! -name "tms*" \
		-and ! -name "private_join_and_compute*"\
		-and ! -name "online_decrypt*"\
		-and ! -name "image_resizing*"\
		-and ! -name "kmeans*"\
		-and ! -name "logistic_reg*"\
		-and ! -name "lin_reg*"\
		-and ! -name "svm*"\
		-and ! -name "gen_linear_model*"\
		-and ! -name "gaussian_mixture_model*"\
		-and ! -name "gaussian_processes*"\
		-and ! -name "dbscan*"\
		-and ! -name "neural_net*"\
		-and ! -name "naive_bayes*"\
		-and ! -name "mesatee_core*" -and ! -name "mesatee_config*" \) \) \
		-exec rm {} \;
	cd $(MODULES_DIR) && \
		for tag in `find $(MESATEE_PROJECT_ROOT) -name sgx_cov*.gcda | cut -d'.' -f2`; \
		do mkdir -p $(OUT_DIR)/cov_$$tag && \
		find target -name *$$tag* -exec mv {} $(OUT_DIR)/cov_$$tag/ \; ; \
		$(LCOV) $(LCOVOPT) --capture \
		--directory $(OUT_DIR)/cov_$$tag/ --base-directory . \
		-o $(OUT_DIR)/modules_$$tag.info; done 2>/dev/null
	rm -rf $(OUT_DIR)/cov_*
	cd $(MODULES_DIR) && $(LCOV) $(LCOVOPT) --capture \
		--directory . --base-directory . \
		-o $(OUT_DIR)/modules.info 2>/dev/null
	cd $(OUT_DIR) && $(LCOV) $(LCOVOPT) $(shell for tag in \
		`find $(MESATEE_PROJECT_ROOT) -name sgx_cov*.gcda | cut -d'.' -f2`; \
		do echo "--add modules_$$tag.info"; done) \
		--add modules.info -o merged.info
	$(LCOV) $(LCOVOPT) --extract $(OUT_DIR)/merged.info \
		`find $(MESATEE_PROJECT_ROOT) -path $(MESATEE_PROJECT_ROOT)/third_party -prune -o \
		-name "*.rs"` -o cov.info
	$(GENHTML) --branch-coverage --demangle-cpp --legend cov.info \
		-o cov_report --ignore-errors source

cov-clean:
	rm -rf $(OUT_DIR)/*.info $(OUT_DIR)/cov_* cov.info cov_report
	find . -name *.gcda -exec rm {} \;

clean: cov-clean
	$(call cargo_clean, $(MODULES_DIR)/mesatee_core)
	$(call sgx_build_clean)
	rm -rf $(MESATEE_BIN_DIR) $(OUT_DIR) $(TARGET_DIR)
	for m in $(SGX_MODULES); do rm -f $$m/pkg_name; done

sgx-test:
ifeq ($(SGX_MODE), HW)
	if [ ! -f bin/ias_spid.txt ] || [ ! -f bin/ias_key.txt ] ; then \
        echo "Please follow \"How to Run (SGX)\" in README to obtain \
ias_spid.txt and ias_key.txt, and put in the bin"; exit 1; fi
endif
	cd tests && ./module_test.sh
	cd tests && ./functional_test.sh
	cd tests && ./integration_test.sh

format: prep
	rustup component add rustfmt --toolchain $(TOOLCHAIN) > /dev/null 2>&1
	RUSTUP_TOOLCHAIN=$(TOOLCHAIN) find $(MESATEE_PROJECT_ROOT) \
		-path $(MESATEE_PROJECT_ROOT)/third_party -prune -o \
		-path $(MESATEE_PROJECT_ROOT)/.git -prune -o \
		-name "*.rs" -exec rustfmt {} +

check: prep
	rustup component add rustfmt --toolchain $(TOOLCHAIN) > /dev/null 2>&1
	RUSTUP_TOOLCHAIN=$(TOOLCHAIN) find $(MESATEE_PROJECT_ROOT) \
		-path $(MESATEE_PROJECT_ROOT)/third_party  -prune -o \
		-path $(MESATEE_PROJECT_ROOT)/.git -prune -o \
		-name "*.rs" -exec rustfmt --check {} +

doc: $(LIBS)
	for m in $(LIBS); do cargo doc -p $$m 2>/dev/null; done

ifndef VERBOSE
.SILENT:
endif

.PHONY: all clean format prep check-sgx-sdk init-submodules
.PHONY: sgx sgx_trusted sgx_untrusted $(SGX_MODULES) sgx_pregen
.PHONY: config_gen unix $(UNIX_MODULES) examples
.PHONY: cov-clean cov-data-clean
