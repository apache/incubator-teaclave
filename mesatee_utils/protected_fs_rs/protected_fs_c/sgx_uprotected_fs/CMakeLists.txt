
if (NOT NON_SGX_PROTECTED_FS)
include_directories("${SGX_SDK}/include")
endif()


add_library(uprotected_fs STATIC 
    sgx_uprotected_fs.cpp 
)

install (TARGETS uprotected_fs DESTINATION bin)