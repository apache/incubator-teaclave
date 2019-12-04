add_custom_target(prep ALL
    COMMAND ${CMAKE_COMMAND} -E env ${TARGET_PREP_ENVS} SGX_UNTRUSTED_CFLAGS=${STR_SGX_UNTRUSTED_CFLAGS}
        SGX_TRUSTED_CFLAGS=${STR_SGX_TRUSTED_CFLAGS} ${MT_SCRIPT_DIR}/prep.sh)

add_custom_target(format
	COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
	COMMAND RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${MESATEE_PROJECT_ROOT}
		-path ${MESATEE_PROJECT_ROOT}/third_party -prune -o
		-path ${MESATEE_PROJECT_ROOT}/.git -prune -o
	 	-path *porst_generated -prune -o
        -name "*.rs" -exec rustfmt {} +
    COMMENT "Formating every .rs file"
    DEPENDS prep
)

add_custom_target(check
	COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
	COMMAND RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${MESATEE_PROJECT_ROOT}
        -path ${MESATEE_PROJECT_ROOT}/third_party -prune -o
        -path ${MESATEE_PROJECT_ROOT}/.git -prune -o
        -path *prost_generated -prune -o
        -name "*.rs" -exec rustfmt --check {} +
    COMMENT "Checking the format of every .rs file"
    DEPENDS prep
)

add_custom_target(sgx-test
    COMMAND ${MESATEE_COMMON_ENVS} ${MT_SCRIPT_DIR}/sgx_test.sh)

add_custom_target(cov
    COMMAND ${MESATEE_COMMON_ENVS} ${MT_SCRIPT_DIR}/gen_cov.sh
)

add_custom_target(cov-clean
    COMMAND rm -rf ${MESATEE_OUT_DIR}/*.info ${MESATEE_OUT_DIR}/cov_* cov.info cov_report
    COMMAND find . -name *.gcda -exec rm {} "\;"
    WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
)

# add folders for "clean" target
set_property(DIRECTORY PROPERTY ADDITIONAL_MAKE_CLEAN_FILES
   "${MESATEE_INSTALL_DIR}"
   "${MESATEE_OUT_DIR}"
   "${MESATEE_TARGET_DIR}"
   "${PROJECT_BINARY_DIR}/cmake_tomls")

# doc target
add_custom_target(doc
    COMMAND cd ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app &&
        ${MESATEE_COMMON_ENVS} cargo doc --all ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS}
    DEPENDS prep ${TARGET_CONFIG_GEN}
)

# add clippy-${sgxlib} targets separately
set(SGXLIB_CLIPPY_TARGETS)
foreach(sgxlib_pkg ${SGXLIB_PKGS})
    add_custom_target(clippy-${sgxlib_pkg}
        COMMAND cd ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_trusted_lib && 
            ${MESATEE_COMMON_ENVS} cargo clippy -p ${sgxlib_pkg} ${CARGO_BUILD_FLAGS} ${SGX_ENCLAVE_FEATURES}
        DEPENDS prep ${TARGET_CONFIG_GEN})
    dbg_message("adding target clippy-${sgxlib_pkg}")
    list(APPEND SGXLIB_CLIPPY_TARGETS clippy-${sgxlib_pkg})
endforeach()
dbg_message("SGXLIB_CLIPPY_TARGETS=${SGXLIB_CLIPPY_TARGETS}")

# clippy target
add_custom_target(clippy
    COMMAND cd ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app && 
        ${MESATEE_COMMON_ENVS} cargo clippy --all ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS}
    COMMAND cd ${PROJECT_BINARY_DIR}/cmake_tomls/sgx_untrusted_app && 
        ${MESATEE_COMMON_ENVS} cargo clippy --all ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS}
    DEPENDS prep ${TARGET_CONFIG_GEN} ${SGXLIB_CLIPPY_TARGETS}
)

gen_convenience_targets()
