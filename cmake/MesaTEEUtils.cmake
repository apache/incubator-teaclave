macro(SET_STRVAR_FROM_ENV_OR var default_val docstring)
if (NOT "$ENV{${var}}" STREQUAL "")
    set(${var} "$ENV{${var}}" CACHE STRING "${docstring}")
else()
    set(${var} "${default_val}" CACHE STRING "${docstring}")
endif()
endmacro()

function(check_sgx_sdk)
    if(NOT IS_DIRECTORY "${SGX_SDK}")
        message(FATAL_ERROR "SGX SDK not found at ${SGX_SDK}, please adjust the SGX_SDK env or the CMake config.")
    endif()
endfunction()

function(init_submodules)
    execute_process (
    COMMAND bash -c "if git submodule status | egrep -q '^[-]|^[+]'; then echo 'INFO: Need to reinitialize git submodules' && git submodule update --init --recursive; fi"
    )
endfunction()

# add_cargo_build_target(package_name
# [TARGET_NAME target_name] # default to cg_${package_name}
# TOML_DIR toml_dir
# TARGET_DIR target_dir
# [DEPENDS [dep]...]
# [NOT_SET_COMMON_ENV]
# [EXTRA_CARGO_FLAGS flg...]
# )
function(add_cargo_build_target package_name)
    set(options NOT_SET_COMMON_ENV)
    set(oneValueArgs TARGET_NAME TOML_DIR TARGET_DIR INSTALL_DIR)
    set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
    cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    if (DEFINED MTEE_TARGET_NAME)
        set(_target_name ${MTEE_TARGET_NAME})
    else()
        set(_target_name cg_${package_name})
    endif()

    if (DEFINED MTEE_INSTALL_DIR)
        set(_copy_dir ${MTEE_INSTALL_DIR})
    else()
        set(_copy_dir ${MESATEE_INSTALL_DIR})
    endif()

    if (MTEE_NOT_SET_COMMON_ENV)
        set(_envs)
    else()
        set(_envs ${MESATEE_COMMON_ENVS})
    endif()

    if (DEFINED MTEE_DEPENDS)
        set(_depends DEPENDS ${MTEE_DEPENDS})
    else()
        set(_depends)
    endif()

    add_custom_target(${_target_name} ALL
        COMMAND ${CMAKE_COMMAND} -E env ${_envs} RUSTFLAGS=${RUSTFLAGS}
            ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${package_name}
            --target-dir ${MTEE_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS}
            && cp ${MTEE_TARGET_DIR}/${TARGET}/${package_name} ${_copy_dir}
        ${_depends}
        COMMENT "Building ${_target_name}"
        WORKING_DIRECTORY ${MTEE_TOML_DIR}
    )
endfunction()

# add_cargo_build_lib_target(package_name
# [TARGET_NAME target_name] # default to cg_${package_name}
# TOML_DIR toml_dir
# TARGET_DIR target_dir
# [DEPENDS [dep]...]
# [NOT_SET_COMMON_ENV]
# [EXTRA_CARGO_FLAGS flg...]
# )
function(add_cargo_build_dylib_target package_name)
    set(options NOT_SET_COMMON_ENV)
    set(oneValueArgs TARGET_NAME TOML_DIR TARGET_DIR)
    set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
    cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    if (DEFINED MTEE_TARGET_NAME)
        set(_target_name ${MTEE_TARGET_NAME})
    else()
        set(_target_name cg_${package_name})
    endif()

    if (MTEE_NOT_SET_COMMON_ENV)
        set(_envs)
    else()
        set(_envs ${MESATEE_COMMON_ENVS})
    endif()

    if (DEFINED MTEE_DEPENDS)
        set(_depends DEPENDS ${MTEE_DEPENDS})
    else()
        set(_depends)
    endif()

    add_custom_target(${_target_name} ALL
        COMMAND ${CMAKE_COMMAND} -E env ${_envs} RUSTFLAGS=${RUSTFLAGS}
            ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${package_name}
            --target-dir ${MTEE_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${MTEE_EXTRA_CARGO_FLAGS}
            && cp ${MTEE_TARGET_DIR}/${TARGET}/lib${package_name}.so ${MESATEE_LIB_INSTALL_DIR}
        ${_depends}
        COMMENT "Building ${_target_name} as a dynamic library"
        WORKING_DIRECTORY ${MTEE_TOML_DIR}
    )
endfunction()

# add_sgx_build_target(sgx_module_path
# [DEPENDS [dep]...]
# [EXTRA_CARGO_FLAGS flg...]
# )
function(add_sgx_build_target sgx_module_path)
    set(options)
    set(oneValueArgs)
    set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
    cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    if (DEFINED MTEE_DEPENDS)
        set(_depends DEPENDS ${MTEE_DEPENDS})
    else()
        set(_depends)
    endif()

    get_filename_component(_module_name ${sgx_module_path} NAME)

    set(_target_name ${SGXLIB_PREFIX}-${_module_name})

    set(package_name)
    string(APPEND package_name ${_module_name} "_enclave")

    add_custom_target(${_target_name} ALL
        COMMAND ${CMAKE_COMMAND} -E env ${MESATEE_COMMON_ENVS} RUSTFLAGS=${RUSTFLAGS}
            ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${package_name}
            --target-dir ${TRUSTED_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${SGX_ENCLAVE_FEATURES} ${MTEE_EXTRA_CARGO_FLAGS}
        COMMAND ${CMAKE_COMMAND} -E env ${TARGET_SGXLIB_ENVS} SGX_COMMON_CFLAGS=${STR_SGX_COMMON_CFLAGS}
            CUR_MODULE_NAME=${_module_name} CUR_MODULE_PATH=${sgx_module_path} ${MT_SCRIPT_DIR}/sgx_link_sign.sh
        ${_depends}
        COMMENT "Building ${_target_name}"
        WORKING_DIRECTORY ${MT_SGXLIB_TOML_DIR}
    )
endfunction()

function(add_enclave_sig_target_n_hooks)
    # add a target to generate enclave sig files
    add_custom_target(update_sig ALL
        COMMAND ${MESATEE_COMMON_ENVS} ${MT_SCRIPT_DIR}/gen_enclave_sig.sh
        COMMENT "Generating enclave signatures..."
        DEPENDS ${SGXLIB_TARGETS}
    )

    # Hook the convenience targets for SGX_MODULES
    # so manually `make kms/tms/...` will trigger
    # updating enclave sig files
    foreach(sgx_module ${SGX_MODULES})
        add_custom_command(TARGET ${sgx_module}
                    POST_BUILD
                    COMMENT "Updating enclave signatures..."
                    COMMAND ${MESATEE_COMMON_ENVS} ${MT_SCRIPT_DIR}/gen_enclave_sig.sh
        )
    endforeach()
endfunction()

function(join_string values glue out)
  string(REGEX REPLACE "([^\\]|^);" "\\1${glue}" _res "${values}")
  string(REGEX REPLACE "[\\](.)" "\\1" _res "${_res}")
  set(${out} "${_res}" PARENT_SCOPE)
endfunction()

function(generate_env_file)
    set(envs ${MESATEE_COMMON_ENVS})
    list(FILTER envs INCLUDE REGEX "MESATEE_PROJECT_ROOT|MESATEE_CFG_DIR|\
MESATEE_BUILD_CFG_DIR|MESATEE_OUT_DIR|MESATEE_AUDITORS_DIR")
    # add extra env vars
    list(APPEND envs "MESATEE_TEST_MODE=1" "RUST_LOG=info" "RUST_BACKTRACE=1")
    join_string("${envs}" "\nexport " env_file)
    string(PREPEND env_file "export ")
    string(APPEND env_file "\n")
    file(WRITE ${PROJECT_BINARY_DIR}/environment ${env_file})
    message(STATUS "====== ${PROJECT_BINARY_DIR}/environment GENERATED ======")
endfunction()

function(gen_convenience_targets)
    # add a target with the same name for each unix_module
    foreach(unix_module ${UNIX_APPS})
        add_custom_target(${unix_module}
            DEPENDS ${UNIXAPP_PREFIX}-${unix_module}
        )
    endforeach()

    # add a target with the same name for each sgx_module (build sgxlib+sgxapp)
    foreach(sgx_module ${SGX_MODULES})
        add_custom_target(${sgx_module}
            DEPENDS ${SGXAPP_PREFIX}-${sgx_module} ${SGXLIB_PREFIX}-${sgx_module}
        )
    endforeach()
endfunction()

function(new_list_with_prefix new_list_name prefix)
    set(_new_list)
    foreach(item ${ARGN})
        string(PREPEND item ${prefix})
        set(_new_list ${_new_list} ${item})
    endforeach()
    set(${new_list_name} ${_new_list} PARENT_SCOPE)
endfunction()

function(new_list_with_suffix new_list_name suffix)
    set(_new_list)
    foreach(item ${ARGN})
        string(APPEND item ${suffix})
        set(_new_list ${_new_list} ${item})
    endforeach()
    set(${new_list_name} ${_new_list} PARENT_SCOPE)
endfunction()

function(new_list_with_insert_prefix new_list_name insert_prefix)
    set(_new_list)
    foreach(item ${ARGN})
        set(_new_list ${_new_list} ${insert_prefix} ${item})
    endforeach()
    set(${new_list_name} ${_new_list} PARENT_SCOPE)
endfunction()


function(check_exe_dependencies)
    foreach(exe ${ARGN})
        execute_process(COMMAND bash -c "type ${exe}"
        OUTPUT_QUIET
        ERROR_QUIET
        RESULT_VARIABLE _res
        )
        if(_res)
            message(FATAL_ERROR "MesaTEE depends on \"${exe}\" but the command was not found. \
Please install the dependency and retry.")
        endif()
    endforeach()
endfunction()

function(parse_cargo_packages packages_name)
    set(options)
    set(oneValueArgs CARGO_TOML_PATH)
    set(multiValueArgs)

    cmake_parse_arguments(MC "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    set(_packages_name)
    set(err)

    execute_process(
        COMMAND python ${PROJECT_SOURCE_DIR}/cmake/scripts/parse_cargo_packages.py
            ${MC_CARGO_TOML_PATH} ${PROJECT_SOURCE_DIR}
        OUTPUT_VARIABLE _packages_name
        ERROR_VARIABLE err
    ) 

    if(NOT (err STREQUAL ""))
        message(FATAL_ERROR "failed to load packages: ${err}")
    endif()

    # level up the local variable to its parent scope
    set(${packages_name} ${_packages_name} PARENT_SCOPE)
endfunction()

# SGXLIB_PKGS, SGXAPP_PKGS, UNIXLIB_PKGS, UNIXAPP_PKGS
# SGXLIB_PKGS_P, SGXAPP_PKGS_P, UNIXLIB_PKGS_P, UNIXAPP_PKGS_P
# _P version is like -p;kms;-p;tms
macro(gen_cargo_package_lists)
    new_list_with_suffix(SGXLIB_PKGS "_enclave" ${SGX_MODULES})
    new_list_with_insert_prefix(SGXLIB_PKGS_P "-p" ${SGXLIB_PKGS})
    set(SGXAPP_PKGS ${SGX_MODULES})
    new_list_with_insert_prefix(SGXAPP_PKGS_P "-p" ${SGXAPP_PKGS})
    set(UNIXLIB_PKGS ${UNIX_LIBS})
    new_list_with_insert_prefix(UNIXLIB_PKGS_P "-p" ${UNIXLIB_PKGS})
    set(UNIXAPP_PKGS ${UNIX_APPS})
    new_list_with_insert_prefix(UNIXAPP_PKGS_P "-p" ${UNIXAPP_PKGS})
endmacro()

macro(dbg_message)
    if(MESATEE_CMAKE_DBG)
        message("${ARGN}")
    endif()
endmacro()
