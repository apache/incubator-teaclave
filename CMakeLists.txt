cmake_minimum_required (VERSION 3.10)
project (MesaTEE C)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(MesaTEEUtils)
include(ExternalProject)
check_exe_dependencies(git openssl python rustup lsof pypy)
init_submodules()

# ====== VARIABLES FOR CMAKE -D{VAR}=VAL CONFIGURATION BEGIN ======
SET_STRVAR_FROM_ENV_OR(SGX_SDK "/opt/sgxsdk" "Path of SGX SDK")
SET_STRVAR_FROM_ENV_OR(SGX_MODE "HW" "Mode of SGX, HW or SW")
SET_STRVAR_FROM_ENV_OR(RUSTFLAGS "" "Rust flags")
SET_STRVAR_FROM_ENV_OR(MESATEE_CMAKE_DBG "" "set to turn on debug message for cmake")
set(MESAPY_VERSION 2b7ea38773d337d106c613a239661a60a4fa7528)
option(COV OFF "Turn on coverage or not")
# ======= VARIABLES FOR CMAKE -D{VAR}=VAL CONFIGURATION END =======

# =============== VARIABLES FOR MANUAL CHANGE BEGIN ===============
set(UNIX_LIBS mesatee_sdk protected_fs_rs)
# need relative path for SGX_MODULE_PATHS to find Enclave.config.xml
set(SGX_MODULE_PATHS mesatee_services/kms mesatee_services/tdfs mesatee_services/tms
    mesatee_services/fns mesatee_services/acs tests/functional_test)
# SGX_MODULES is generated from SGX_MODULE_PATHS
# set(SGX_MODULES kms tdfs tms fns acs functional_test)
# ================ VARIABLES FOR MANUAL CHANGE END ================

# AUTO-GENERATE VARS
## parse the unix apps to build
parse_cargo_packages(UNIX_APPS
    CARGO_TOML_PATH ${PROJECT_SOURCE_DIR}/cmake/tomls/Cargo.unix_app.toml
)

include(MesaTEEGenVars)

# ========== CONFIG-TIME CHECKS AND INITIALIZATIONS BEGIN =========
check_sgx_sdk()
generate_env_file()
# =========== CONFIG-TIME CHECKS AND INITIALIZATIONS END ==========

# UTIL TARGETS: prep, sgx-test, cov, cov-clean, format, check, doc, etc.
include(UtilTargets)

# unix_app
foreach(package_name ${UNIX_APPS})
    add_cargo_build_target(${package_name}
        TARGET_NAME "${UNIXAPP_PREFIX}-${package_name}"
        TOML_DIR ${MT_UNIX_TOML_DIR}
        TARGET_DIR ${UNIX_TARGET_DIR}
        INSTALL_DIR ${MESATEE_EXAMPLE_INSTALL_DIR}
        DEPENDS prep ${TARGET_CONFIG_GEN}
    )
endforeach()

# sgx_untrusted_app
foreach(package_name ${SGX_MODULES})
    add_cargo_build_target(${package_name}
        TARGET_NAME "${SGXAPP_PREFIX}-${package_name}"
        TOML_DIR ${MT_SGXAPP_TOML_DIR}
        TARGET_DIR ${UNTRUSTED_TARGET_DIR}
        INSTALL_DIR ${MESATEE_SERVICE_INSTALL_DIR}
        DEPENDS prep ${TARGET_CONFIG_GEN}
    )
endforeach()

# mesapy components
add_custom_command(
    OUTPUT ${MESATEE_OUT_DIR}/acs_py_enclave.c
    COMMAND env
    ARGS PYTHONPATH=${PROJECT_SOURCE_DIR}/third_party/mesapy/sgx
         PYPY_FFI_OUTDIR=${MESATEE_OUT_DIR}
         pypy ${PROJECT_SOURCE_DIR}/mesatee_services/acs/python/ffi.py
    DEPENDS prep
            ${PROJECT_SOURCE_DIR}/mesatee_services/acs/python/ffi.py
            ${PROJECT_SOURCE_DIR}/mesatee_services/acs/python/acs_engine.py
    COMMENT "Generating mesapy ffi stubs"
)
add_library(pycomponent STATIC ${MESATEE_OUT_DIR}/acs_py_enclave.c)
set_target_properties(pycomponent PROPERTIES ARCHIVE_OUTPUT_DIRECTORY ${MESATEE_OUT_DIR})
target_compile_definitions(pycomponent PUBLIC SGX)
if(NOT EXISTS "/usr/lib/pypy/include")
    message(FATAL_ERROR "pypy development package not found\nFor Ubuntu, please run `apt-get install pypy-dev`")
endif()
target_compile_options(pycomponent PUBLIC -UWITH_THREAD -O2 -fPIC -Wimplicit -I/usr/lib/pypy/include)


# sgx_trusted_lib
foreach(package_path ${SGX_MODULE_PATHS})
    set(_package_name)
    get_filename_component(_package_name ${package_path} NAME)
    add_sgx_build_target(${package_path}
        DEPENDS prep ${TARGET_CONFIG_GEN} "${SGXAPP_PREFIX}-${_package_name}" pycomponent
    )
endforeach()

# MesaTEE C SDK
add_cargo_build_dylib_target(mesatee_sdk_c
    TARGET_NAME "${UNIXLIB_PREFIX}-mesatee_sdk_c"
    TOML_DIR ${MT_UNIX_TOML_DIR}
    TARGET_DIR ${UNIX_TARGET_DIR}
    DEPENDS prep ${TARGET_CONFIG_GEN}
)

# example/quickstart_c
link_directories(${MESATEE_LIB_INSTALL_DIR})
add_executable(quickstart_c ${MESATEE_PROJECT_ROOT}/examples/quickstart_c/main.c)
add_dependencies(quickstart_c prep ${TARGET_CONFIG_GEN} ${UNIXLIB_PREFIX}-mesatee_sdk_c)
target_include_directories(quickstart_c
    PUBLIC ${MESATEE_PROJECT_ROOT}/mesatee_sdk/c_sdk/include/)
target_link_libraries(quickstart_c mesatee_sdk_c)
add_custom_command(TARGET quickstart_c POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_CURRENT_BINARY_DIR}/quickstart_c
    ${MESATEE_EXAMPLE_INSTALL_DIR}/quickstart_c
)

add_enclave_sig_target_n_hooks()
