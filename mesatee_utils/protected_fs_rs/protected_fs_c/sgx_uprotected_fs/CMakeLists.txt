if (NOT NON_SGX_PROTECTED_FS)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${SGX_COMMON_CFLAGS} -fPIC -Wno-attributes")
include_directories("${SGX_SDK}/include")
endif()

add_library(uprotected_fs STATIC 
    sgx_uprotected_fs.cpp 
)

install (TARGETS uprotected_fs DESTINATION bin)