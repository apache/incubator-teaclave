DEBUG			?= 0
CARGO_BUILD_ARGS 	= cargo +nightly build
CARGO_RUN_ARGS 		= cargo +nightly run
# Enable modular executor library loaded at runtime.
MODULAR			?= 1
LIB_PATH		?= $(CURDIR)/../../lib

ifeq ($(DEBUG), 0)
	CARGO_BUILD_ARGS += --release
	CARGO_RUN_ARGS += --release
	CARGO_OUTPUT_PATH = target/release
else
	CARGO_OUTPUT_PATH = target/debug
endif

ifeq ($(MODULAR), 1)
	CARGO_FEATURES += modular
	LIB_NAME = $(LIB_PATH)/libexecutor_lib.so
else
	LIB_NAME = $(LIB_PATH)/libexecutor_lib.a
	export USE_STATIC = true
	export STATIC_LIB_PATH = $(LIB_PATH)
	export STATIC_LIB_NAME = executor_lib
endif

.phony: all clean

all: run

$(LIB_NAME):
	@mkdir -p $(LIB_PATH)
	@$(CARGO_BUILD_ARGS) -p executor_lib --features=$(CARGO_FEATURES)
	cp $(CARGO_OUTPUT_PATH)/libexecutor_lib* $(LIB_PATH)

build: $(LIB_NAME)
	@$(CARGO_BUILD_ARGS) --features=$(CARGO_FEATURES)

clean:
	@cargo clean
	@cargo clean
	@rm -rf lib

run: $(LIB_NAME) build
	@$(CARGO_RUN_ARGS) --features=$(CARGO_FEATURES) --bin executor_user
