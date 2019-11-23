cmake_minimum_required (VERSION 2.6)
project(protected_fs_c)
set (PROTECTED_FS_VERSION_MAJOR 0)
set (PROTECTED_FS_VERSION_MINOR 1)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/target)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/target)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/target)

configure_file(
    "protected_fs_config.h.in"
    "${PROJECT_BINARY_DIR}/protected_fs_config.h"
)
include_directories("${PROJECT_BINARY_DIR}")
include_directories("inc")

option (NON_SGX_PROTECTED_FS
        "Use PROTECTED_FS NON SGX VERSION" ON)

set (SGX_SDK $ENV{SGX_SDK})

if (NON_SGX_PROTECTED_FS)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Werror -U__STRICT_ANSI__ -std=c++11 -lpthread -fPIC")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unused-local-typedefs -Wno-shadow -Wno-missing-field-initializers -Wno-unused-parameter")
elseif (NOT EXISTS ${SGX_SDK})
    message(FATAL_ERROR "$SGX_SDK directory must exist")
else ()
    message("SGX_SDK=${SGX_SDK}")

    set(SGX_LIBRARY_PATH  ${SGX_SDK}/lib64)
    set(SGX_COMMON_CFLAGS  -m64 -O2)
    set(SGX_UNTRUSTED_CFLAGS  ${SGX_COMMON_CFLAGS} -fPIC -Wno-attributes
        -I${SGX_SDK}/include)
    set(SGX_TRUSTED_CFLAGS  ${SGX_COMMON_CFLAGS} -nostdinc -fvisibility=hidden
        -fpie -fstack-protector
        -I${SGX_SDK}/include -I${SGX_SDK}/include/tlibc
        -I${SGX_SDK}/include/stdc++ -I${SGX_SDK}/include/libcxx 
        -I${SGX_SDK}/include/stlport -I${SGX_SDK}/include/epid)

    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -U__STRICT_ANSI__ -Werror")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unused-local-typedefs -Wno-shadow -Wno-missing-field-initializers -Wno-unused-parameter")
    set(${PROJECT_NAME}_sgx_tprotected_fs_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${SGX_TRUSTED_CFLAGS}")
    set(${PROJECT_NAME}_sgx_uprotected_fs_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${SGX_UNTRUSTED_CFLAGS}")
endif (NON_SGX_PROTECTED_FS)

add_subdirectory (sgx_tprotected_fs)
add_subdirectory (sgx_uprotected_fs)