set(MESATEE_PROJECT_ROOT ${PROJECT_SOURCE_DIR})
set(MESATEE_BUILD_ROOT ${PROJECT_BINARY_DIR})
set(MESATEE_OUT_DIR ${PROJECT_BINARY_DIR}/intermediate)
set(MESATEE_INSTALL_DIR ${PROJECT_SOURCE_DIR}/release)
set(MESATEE_SERVICE_INSTALL_DIR ${MESATEE_INSTALL_DIR}/service)
set(MESATEE_EXAMPLE_INSTALL_DIR ${MESATEE_INSTALL_DIR}/example)
set(MESATEE_LIB_INSTALL_DIR ${MESATEE_INSTALL_DIR}/lib)
set(MESATEE_AUDITORS_DIR ${MESATEE_SERVICE_INSTALL_DIR}/auditors)
set(MESATEE_TARGET_DIR ${PROJECT_BINARY_DIR}/target)

set(TOOLCHAIN_DEPS_DIR ${PROJECT_SOURCE_DIR}/toolchain_deps)
set(THIRD_PARTY_DIR ${PROJECT_SOURCE_DIR}/third_party)
set(UNTRUSTED_TARGET_DIR ${MESATEE_TARGET_DIR}/untrusted)
set(UNIX_TARGET_DIR ${MESATEE_TARGET_DIR}/unix)
set(TRUSTED_TARGET_DIR ${MESATEE_TARGET_DIR}/trusted)
# build.rs will read ENV{ENCLAVE_OUT_DIR} for linking
set(ENCLAVE_OUT_DIR ${MESATEE_OUT_DIR})
set(RUST_SGX_SDK ${PROJECT_SOURCE_DIR}/third_party/rust-sgx-sdk)
set(MT_SCRIPT_DIR ${PROJECT_SOURCE_DIR}/cmake/scripts)
set(MT_UNIX_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app)
set(MT_SGXLIB_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_trusted_lib)
set(MT_SGXAPP_TOML_DIR ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_untrusted_app)
set(MT_EDL_FILE ${PROJECT_SOURCE_DIR}/mesatee_binder/Enclave.edl)

set(SGX_EDGER8R ${SGX_SDK}/bin/x64/sgx_edger8r)
set(SGX_ENCLAVE_SIGNER  ${SGX_SDK}/bin/x64/sgx_sign)
set(SGX_LIBRARY_PATH  ${SGX_SDK}/lib64)

set(SGX_COMMON_CFLAGS  -m64 -O2)
set(SGX_UNTRUSTED_CFLAGS  ${SGX_COMMON_CFLAGS} -fPIC -Wno-attributes
       -I${SGX_SDK}/include -I${RUST_SGX_SDK}/edl)
set(SGX_TRUSTED_CFLAGS  ${SGX_COMMON_CFLAGS} -nostdinc -fvisibility=hidden
       -fpie -fstack-protector
       -I${RUST_SGX_SDK}/edl -I${RUST_SGX_SDK}/common/inc
       -I${SGX_SDK}/include -I${SGX_SDK}/include/tlibc
    -I${SGX_SDK}/include/stlport -I${SGX_SDK}/include/epid)
join_string("${SGX_COMMON_CFLAGS}" " " STR_SGX_COMMON_CFLAGS)
join_string("${SGX_UNTRUSTED_CFLAGS}" " " STR_SGX_UNTRUSTED_CFLAGS)
join_string("${SGX_TRUSTED_CFLAGS}" " " STR_SGX_TRUSTED_CFLAGS)

if (NOT "${SGX_MODE}" STREQUAL "HW")
	set(Trts_Library_Name sgx_trts_sim)
	set(Service_Library_Name sgx_tservice_sim)
else()
	set(Trts_Library_Name sgx_trts)
	set(Service_Library_Name sgx_tservice)
endif()

set(SGX_ENCLAVE_FEATURES -Z package-features --features mesalock_sgx)
string(TOLOWER "${CMAKE_BUILD_TYPE}" CMAKE_BUILD_TYPE_LOWER)
if (CMAKE_BUILD_TYPE_LOWER STREQUAL "debug")
    set(TARGET debug)
    set(CARGO_BUILD_FLAGS "")

    if (COV)
        check_exe_dependencies(lcov llvm-cov)
        set(SGX_ENCLAVE_FEATURES -Z package-features --features "mesalock_sgx cov")
        set(CARGO_INCREMENTAL 0)
        set(RUSTFLAGS "-D warnings -Zprofile -Ccodegen-units=1 \
-Cllvm_args=-inline-threshold=0 -Coverflow-checks=off -Zno-landing-pads")
    endif()
else()
    set(TARGET release)
    set(CARGO_BUILD_FLAGS --release)
endif()

execute_process (
    COMMAND bash -c "cat ${PROJECT_SOURCE_DIR}/third_party/rust-sgx-sdk/rust-toolchain"
    OUTPUT_VARIABLE RUSTUP_TOOLCHAIN
    )
string(STRIP ${RUSTUP_TOOLCHAIN} RUSTUP_TOOLCHAIN)
set(RUSTUP_TOOLCHAIN ${RUSTUP_TOOLCHAIN})

set(UNIXAPP_PREFIX "unixapp")
set(UNIXLIB_PREFIX "unixlib")
set(SGXAPP_PREFIX "sgxapp")
set(SGXLIB_PREFIX "sgxlib")
set(TARGET_CONFIG_GEN "${UNIXAPP_PREFIX}-config_gen")

# generate SGX_MODULES from SGX_MODULE_PATHS
set(SGX_MODULES)
foreach(sgx_module_path ${SGX_MODULE_PATHS})
    get_filename_component(_tmp_sgx_module ${sgx_module_path} NAME)
    list(APPEND SGX_MODULES ${_tmp_sgx_module})
endforeach()

# generate SGXLIB_TARGETS (sgxlib-fns sgxlib-kms ...)
new_list_with_prefix(SGXLIB_TARGETS "${SGXLIB_PREFIX}-" ${SGX_MODULES})

# generate SGXAPP_TARGETS (sgxapp-fns sgxapp-kms ...)
new_list_with_prefix(SGXAPP_TARGETS "${SGXAPP_PREFIX}-" ${SGX_MODULES})

# generate UNIXAPP_TARGETS (unixapp-config_gen ...)
new_list_with_prefix(UNIXAPP_TARGETS "${UNIXAPP_PREFIX}-" ${UNIX_APPS})

# SGXLIB_PKGS, SGXAPP_PKGS, UNIXLIB_PKGS, UNIXAPP_PKGS
# SGXLIB_PKGS_P, SGXAPP_PKGS_P, UNIXLIB_PKGS_P, UNIXAPP_PKGS_P
# _P version is like -p;kms;-p;tms
gen_cargo_package_lists()

dbg_message("SGXLIB_PKGS_P=${SGXLIB_PKGS_P}")
dbg_message("SGXAPP_PKGS_P=${SGXAPP_PKGS_P}")
dbg_message("UNIXLIB_PKGS_P=${UNIXLIB_PKGS_P}")
dbg_message("UNIXAPP_PKGS_P=${UNIXAPP_PKGS_P}")

set(MESATEE_COMMON_ENVS
    MESATEE_PROJECT_ROOT=${MESATEE_PROJECT_ROOT}
    MESATEE_BUILD_ROOT=${MESATEE_BUILD_ROOT}
    MESATEE_OUT_DIR=${MESATEE_OUT_DIR}
    MESATEE_SERVICE_INSTALL_DIR=${MESATEE_SERVICE_INSTALL_DIR}
    MESATEE_EXAMPLE_INSTALL_DIR=${MESATEE_EXAMPLE_INSTALL_DIR}
    MESATEE_LIB_INSTALL_DIR=${MESATEE_LIB_INSTALL_DIR}
    MESATEE_TARGET_DIR=${MESATEE_TARGET_DIR}
    MESATEE_AUDITORS_DIR=${MESATEE_AUDITORS_DIR}
    MESATEE_CFG_DIR=${PROJECT_SOURCE_DIR}
    MESATEE_BUILD_CFG_DIR=${PROJECT_SOURCE_DIR}
    SGX_SDK=${SGX_SDK}
    SGX_MODE=${SGX_MODE}
    ENCLAVE_OUT_DIR=${ENCLAVE_OUT_DIR}
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}
    RUST_SGX_SDK=${RUST_SGX_SDK}
    MT_SCRIPT_DIR=${MT_SCRIPT_DIR}
    CARGO_INCREMENTAL=${CARGO_INCREMENTAL}
    CMAKE_C_COMPILER=${CMAKE_C_COMPILER}
    CC=${MT_SCRIPT_DIR}/cc_wrapper.sh
    MT_RUSTC_WRAPPER=${MT_SCRIPT_DIR}/rustc_wrapper.sh
)

set(TARGET_PREP_ENVS
${MESATEE_COMMON_ENVS}
CMAKE_SOURCE_DIR=${CMAKE_SOURCE_DIR}
CMAKE_BINARY_DIR=${CMAKE_BINARY_DIR}
MESAPY_VERSION=${MESAPY_VERSION}
SGX_EDGER8R=${SGX_EDGER8R}
MT_EDL_FILE=${MT_EDL_FILE}
CMAKE_AR=${CMAKE_AR}
)

set(TARGET_SGXLIB_ENVS
${MESATEE_COMMON_ENVS}
SGX_LIBRARY_PATH=${SGX_LIBRARY_PATH}
SGX_ENCLAVE_SIGNER=${SGX_ENCLAVE_SIGNER}
Service_Library_Name=${Service_Library_Name}
Trts_Library_Name=${Trts_Library_Name}
TRUSTED_TARGET_DIR=${TRUSTED_TARGET_DIR}
TARGET=${TARGET}
TOOLCHAIN_DEPS_DIR=${TOOLCHAIN_DEPS_DIR}
)

message("SGX_SDK=${SGX_SDK}")
message("SGX_MODE=${SGX_MODE}")
message("RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}")
message("BUILD TYPE=${TARGET}")
