# for common header sgx_tprotected_fs.h
include_directories("${PROJECT_SOURCE_DIR}")

if (NOT NON_SGX_PROTECTED_FS)
include_directories("${SGX_SDK}/include")
include_directories("${SGX_SDK}/include/tlibc")
include_directories("${SGX_SDK}/include/libcxx")
endif()

add_library(tprotected_fs STATIC 
    file_crypto.cpp
    file_flush.cpp
    file_init.cpp
    file_other.cpp
    file_read_write.cpp
    file_version.cpp
    lru_cache.cpp
    sgx_tprotected_fs.cpp 
    non_sgx_protected_fs.cpp
)

install (TARGETS tprotected_fs DESTINATION bin)
