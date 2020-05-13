add_custom_target(
  prep ALL
  COMMAND
    ${CMAKE_COMMAND} -E env ${TARGET_PREP_ENVS}
    SGX_UNTRUSTED_CFLAGS=${STR_SGX_UNTRUSTED_CFLAGS}
    SGX_TRUSTED_CFLAGS=${STR_SGX_TRUSTED_CFLAGS} ${MT_SCRIPT_DIR}/prep.sh)

add_custom_target(
  format
  COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
  COMMAND
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${TEACLAVE_PROJECT_ROOT} -path
    ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o -path
    ${TEACLAVE_PROJECT_ROOT}/.git -prune -o -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.rs" -exec rustfmt {} +
  COMMENT "Formating every .rs file"
  DEPENDS prep)

add_custom_target(
  check
  COMMAND rustup component add rustfmt --toolchain ${RUSTUP_TOOLCHAIN}
  COMMAND
    RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN} find ${TEACLAVE_PROJECT_ROOT} -path
    ${TEACLAVE_PROJECT_ROOT}/third_party -prune -o -path
    ${TEACLAVE_PROJECT_ROOT}/.git -prune -o -path ${TEACLAVE_BUILD_ROOT} -prune
    -o -name "*.rs" -exec rustfmt --check {} +
  COMMENT "Checking the format of every .rs file"
  DEPENDS prep)

if(TEST_MODE)
  add_custom_target(run-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                      ${MT_SCRIPT_DIR}/test.sh)
  add_custom_target(run-unit-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                           ${MT_SCRIPT_DIR}/test.sh unit)
  add_custom_target(
    run-integration-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                  ${MT_SCRIPT_DIR}/test.sh integration)
  add_custom_target(
    run-functional-tests COMMAND ${TEACLAVE_COMMON_ENVS}
                                 ${MT_SCRIPT_DIR}/test.sh functional)
  add_custom_target(run-examples COMMAND ${TEACLAVE_COMMON_ENVS}
                                 ${MT_SCRIPT_DIR}/test.sh example)
else()
  add_custom_target(
    run-tests
    COMMAND
      echo
      "Note: Testing is not enabled in this build. Run cmake again with -DTEST_MODE=ON"
  )
endif()

add_custom_target(cov COMMAND ${TEACLAVE_COMMON_ENVS}
                              ${MT_SCRIPT_DIR}/gen_cov.sh)

add_custom_target(
  cov-clean
  COMMAND rm -rf ${TEACLAVE_OUT_DIR}/*.info ${TEACLAVE_OUT_DIR}/cov_* cov.info
          cov_report
  COMMAND find . -name *.gcda -exec rm {} "\;"
  WORKING_DIRECTORY ${PROJECT_SOURCE_DIR})

# add folders for "clean" target
set_property(
  DIRECTORY
  PROPERTY ADDITIONAL_MAKE_CLEAN_FILES "${TEACLAVE_INSTALL_DIR}"
           "${TEACLAVE_OUT_DIR}" "${TEACLAVE_TARGET_DIR}"
           "${PROJECT_BINARY_DIR}/cmake_tomls")

# doc target
add_custom_target(
  doc
  COMMAND
    cd ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app && ${TEACLAVE_COMMON_ENVS}
    cargo doc --all ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS} && cp -RT
    ${PROJECT_BINARY_DIR}/cmake_tomls/unix_app/target/doc
    ${TEACLAVE_DOC_INSTALL_DIR}
  DEPENDS prep)

# clippy target
add_custom_target(clippy COMMAND make CLP=1 all)

gen_convenience_targets()
