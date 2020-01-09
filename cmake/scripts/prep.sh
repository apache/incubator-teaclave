#!/bin/bash
set -e
REQUIRED_ENVS=("CMAKE_SOURCE_DIR" "CMAKE_BINARY_DIR"
"MESATEE_OUT_DIR" "MESATEE_TARGET_DIR" "RUSTUP_TOOLCHAIN" "MESAPY_VERSION"
"SGX_EDGER8R" "MT_EDL_FILE" "SGX_SDK" "RUST_SGX_SDK" "CMAKE_C_COMPILER"
"CMAKE_AR" "SGX_UNTRUSTED_CFLAGS" "SGX_TRUSTED_CFLAGS" "MT_SCRIPT_DIR"
"MESATEE_SERVICE_INSTALL_DIR" "MESATEE_EXAMPLE_INSTALL_DIR" "MESATEE_BIN_INSTALL_DIR"
"MESATEE_LIB_INSTALL_DIR" "MESATEE_TEST_INSTALL_DIR"
"MESATEE_AUDITORS_DIR" "MESATEE_EXAMPLE_AUDITORS_DIR"
)

for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

${MT_SCRIPT_DIR}/setup_cmake_tomls ${CMAKE_SOURCE_DIR} ${CMAKE_BINARY_DIR}
mkdir -p ${MESATEE_OUT_DIR} ${MESATEE_TARGET_DIR} ${MESATEE_SERVICE_INSTALL_DIR} \
    ${MESATEE_EXAMPLE_INSTALL_DIR} ${MESATEE_BIN_INSTALL_DIR} ${MESATEE_LIB_INSTALL_DIR} \
    ${MESATEE_TEST_INSTALL_DIR} ${MESATEE_AUDITORS_DIR} ${MESATEE_EXAMPLE_AUDITORS_DIR}
# copy auditors to install directory to make it easy to package all built things
cp -RT ${CMAKE_SOURCE_DIR}/keys/auditors/ ${MESATEE_AUDITORS_DIR}/
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${MESATEE_SERVICE_INSTALL_DIR}
cp ${CMAKE_SOURCE_DIR}/config/runtime.config.toml ${MESATEE_TEST_INSTALL_DIR}
cp -r ${CMAKE_SOURCE_DIR}/tests/test_cases/ ${MESATEE_SERVICE_INSTALL_DIR}
cp -r ${CMAKE_SOURCE_DIR}/tests/test_cases/ ${MESATEE_TEST_INSTALL_DIR}
# create the following symlinks to make remapped paths accessible and avoid repeated building
mkdir -p /tmp/mesatee_symlinks
ln -snf ${HOME}/.cargo /tmp/mesatee_symlinks/cargo_home
ln -snf ${CMAKE_SOURCE_DIR} /tmp/mesatee_symlinks/mesatee_src
ln -snf ${CMAKE_BINARY_DIR} /tmp/mesatee_symlinks/mesatee_build
if git submodule status | egrep -q '^[-]|^[+]'; then echo 'INFO: Need to reinitialize git submodules' && git submodule update --init --recursive; fi
rustup install --no-self-update ${RUSTUP_TOOLCHAIN} > /dev/null 2>&1
# get mesapy
if [ ! -f ${MESATEE_OUT_DIR}/libpypy-c.a ] || [ ! -f ${MESATEE_OUT_DIR}/${MESAPY_VERSION}-mesapy-sgx.tar.gz ]; then
    cd ${MESATEE_OUT_DIR};
    echo "Downloading MesaPy ${MESAPY_VERSION}..."
    wget -qN https://mesapy.org/release/${MESAPY_VERSION}-mesapy-sgx.tar.gz;
    tar xzf ${MESAPY_VERSION}-mesapy-sgx.tar.gz;
    cd -
fi
# build libEnclave_u.a & libEnclave_t.o
if [ ! -f ${MESATEE_OUT_DIR}/libEnclave_u.a ]; then
    echo 'INFO: Start to build EDL.'
    ${SGX_EDGER8R} --untrusted ${MT_EDL_FILE} --search-path ${SGX_SDK}/include \
        --search-path ${RUST_SGX_SDK}/edl --untrusted-dir ${MESATEE_OUT_DIR}
    cd ${MESATEE_OUT_DIR}
    ${CMAKE_C_COMPILER} ${SGX_UNTRUSTED_CFLAGS} -c Enclave_u.c -o libEnclave_u.o
    ${CMAKE_AR} rcsD libEnclave_u.a libEnclave_u.o
    ${SGX_EDGER8R} --trusted ${MT_EDL_FILE} --search-path ${SGX_SDK}/include \
        --search-path ${RUST_SGX_SDK}/edl --trusted-dir ${MESATEE_OUT_DIR}
    ${CMAKE_C_COMPILER} ${SGX_TRUSTED_CFLAGS} -c Enclave_t.c -o libEnclave_t.o
fi
