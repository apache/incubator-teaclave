#!/bin/bash
set -e
REQUIRED_ENVS=("CMAKE_SOURCE_DIR" "CMAKE_BINARY_DIR"
"TEACLAVE_OUT_DIR" "TEACLAVE_TARGET_DIR" "RUSTUP_TOOLCHAIN" "MESAPY_VERSION"
"SGX_EDGER8R" "MT_EDL_FILE" "SGX_SDK" "RUST_SGX_SDK" "CMAKE_C_COMPILER"
"CMAKE_AR" "SGX_UNTRUSTED_CFLAGS" "SGX_TRUSTED_CFLAGS" "MT_SCRIPT_DIR"
"TEACLAVE_SERVICE_INSTALL_DIR" "TEACLAVE_EXAMPLE_INSTALL_DIR" "TEACLAVE_BIN_INSTALL_DIR"
"TEACLAVE_CLI_INSTALL_DIR" "TEACLAVE_DCAP_INSTALL_DIR" "TEACLAVE_LIB_INSTALL_DIR" "TEACLAVE_TEST_INSTALL_DIR"
"TEACLAVE_AUDITORS_DIR" "TEACLAVE_EXAMPLE_AUDITORS_DIR" "DCAP" "TEACLAVE_SYMLINKS"
"TEACLAVE_PROJECT_ROOT"
)

for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

${MT_SCRIPT_DIR}/setup_cmake_tomls ${CMAKE_SOURCE_DIR} ${CMAKE_BINARY_DIR}
mkdir -p ${TEACLAVE_OUT_DIR} ${TEACLAVE_TARGET_DIR} ${TEACLAVE_SERVICE_INSTALL_DIR} \
      ${TEACLAVE_EXAMPLE_INSTALL_DIR} ${TEACLAVE_CLI_INSTALL_DIR} \
      ${TEACLAVE_BIN_INSTALL_DIR} ${TEACLAVE_LIB_INSTALL_DIR} \
    ${TEACLAVE_TEST_INSTALL_DIR} ${TEACLAVE_AUDITORS_DIR} ${TEACLAVE_EXAMPLE_AUDITORS_DIR}
if [ -n "$DCAP" ]; then
    mkdir -p ${TEACLAVE_DCAP_INSTALL_DIR}
    cp ${CMAKE_SOURCE_DIR}/dcap/Rocket.toml ${TEACLAVE_DCAP_INSTALL_DIR}/Rocket.toml
    cp ${CMAKE_SOURCE_DIR}/keys/dcap_server_cert.pem ${TEACLAVE_DCAP_INSTALL_DIR}/
    cp ${CMAKE_SOURCE_DIR}/keys/dcap_server_key.pem ${TEACLAVE_DCAP_INSTALL_DIR}/
fi
# copy auditors to install directory to make it easy to package all built things
cp -RT ${CMAKE_SOURCE_DIR}/keys/auditors/ ${TEACLAVE_AUDITORS_DIR}/
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${TEACLAVE_SERVICE_INSTALL_DIR}
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${TEACLAVE_TEST_INSTALL_DIR}
cp -r ${CMAKE_SOURCE_DIR}/tests/fixtures/ ${TEACLAVE_TEST_INSTALL_DIR}
ln -f -s ${TEACLAVE_TEST_INSTALL_DIR}/fixtures ${TEACLAVE_SERVICE_INSTALL_DIR}/fixtures
cp -r ${CMAKE_SOURCE_DIR}/tests/scripts/ ${TEACLAVE_TEST_INSTALL_DIR}
# create the following symlinks to make remapped paths accessible and avoid repeated building
mkdir -p ${TEACLAVE_SYMLINKS}
ln -snf ${HOME}/.cargo ${TEACLAVE_SYMLINKS}/cargo_home
ln -snf ${CMAKE_SOURCE_DIR} ${TEACLAVE_SYMLINKS}/teaclave_src
ln -snf ${CMAKE_BINARY_DIR} ${TEACLAVE_SYMLINKS}/teaclave_build
# cleanup sgx_unwind/libunwind
(cd ${CMAKE_SOURCE_DIR}/third_party/crates-sgx/ && git clean -fdx vendor/sgx_unwind/libunwind/)
if git submodule status | egrep -q '^[-]|^[+]'; then echo 'INFO: Need to reinitialize git submodules' && git submodule update --init --recursive; fi
rustup install --no-self-update ${RUSTUP_TOOLCHAIN} > /dev/null 2>&1
# get mesapy
if [ ! -f ${TEACLAVE_OUT_DIR}/libpypy-c.a ] || [ ! -f ${TEACLAVE_OUT_DIR}/${MESAPY_VERSION}-mesapy-sgx.tar.gz ]; then
    cd ${TEACLAVE_OUT_DIR};
    echo "Downloading MesaPy ${MESAPY_VERSION}..."
    wget -qN https://mesapy.org/release/${MESAPY_VERSION}-mesapy-sgx.tar.gz;
    tar xzf ${MESAPY_VERSION}-mesapy-sgx.tar.gz;
    cd -
fi
# build edl_libs
if [ ! -f ${TEACLAVE_OUT_DIR}/libEnclave_common_u.a ]; then
    echo 'INFO: Start to build EDL.'
    ${SGX_EDGER8R} --untrusted ${MT_EDL_FILE} --search-path ${SGX_SDK}/include \
        --search-path ${RUST_SGX_SDK}/edl --search-path ${TEACLAVE_PROJECT_ROOT}/edl \
        --untrusted-dir ${TEACLAVE_OUT_DIR}
    cd ${TEACLAVE_OUT_DIR}
    ${CMAKE_C_COMPILER} ${SGX_UNTRUSTED_CFLAGS} -c Enclave_common_u.c -o libEnclave_common_u.o
    ${CMAKE_AR} rcsD libEnclave_common_u.a libEnclave_common_u.o

    ${CMAKE_C_COMPILER} ${SGX_UNTRUSTED_CFLAGS} -c Enclave_fa_u.c -o libEnclave_fa_u.o
    ${CMAKE_AR} rcsD libEnclave_fa_u.a libEnclave_fa_u.o

    ${SGX_EDGER8R} --trusted ${MT_EDL_FILE} --search-path ${SGX_SDK}/include \
        --search-path ${RUST_SGX_SDK}/edl --search-path ${TEACLAVE_PROJECT_ROOT}/edl \
        --trusted-dir ${TEACLAVE_OUT_DIR}
    ${CMAKE_C_COMPILER} ${SGX_TRUSTED_CFLAGS} -c Enclave_common_t.c -o libEnclave_common_t.o
    ${CMAKE_C_COMPILER} ${SGX_TRUSTED_CFLAGS} -c Enclave_fa_t.c -o libEnclave_fa_t.o
fi
