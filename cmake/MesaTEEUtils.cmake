macro(dbg_message)
if(MESATEE_CMAKE_DBG)
    message("${ARGN}")
endif()
endmacro()

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

macro(sgxlib_pkgname_2_modname pkg_name mod_name)
    string(REGEX REPLACE "_enclave$" "" ${mod_name} ${pkg_name})
endmacro()

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
    set(oneValueArgs TARGET_NAME TOML_DIR TARGET_DIR INSTALL_DIR EXTRA_CARGO_FLAGS)
    set(multiValueArgs DEPENDS)
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

# add_sgx_build_target(sgx_lib_path
# [DEPENDS [dep]...]
# [INSTALL_DIR dir]
# [EXTRA_CARGO_FLAGS flg...]
# )
function(add_sgx_build_target sgx_lib_path pkg_name)
    set(options)
    set(oneValueArgs INSTALL_DIR)
    set(multiValueArgs DEPENDS EXTRA_CARGO_FLAGS)
    cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    if (DEFINED MTEE_DEPENDS)
        set(_depends DEPENDS ${MTEE_DEPENDS})
    else()
        set(_depends)
    endif()

    if (DEFINED MTEE_INSTALL_DIR)
        set(_copy_dir ${MTEE_INSTALL_DIR})
    else()
        set(_copy_dir ${MESATEE_INSTALL_DIR})
    endif()

    # remove trailing "_enclave" to get _module_name
    sgxlib_pkgname_2_modname(${pkg_name} _module_name)

    set(_target_name ${SGXLIB_PREFIX}-${_module_name})

    if(_module_name STREQUAL "functional_test")
        set(_enclave_info "/dev/null")
    else()
        set(_enclave_info "${MESATEE_OUT_DIR}/${_module_name}_enclave_info.toml")
    endif()

    add_custom_target(${_target_name} ALL
        COMMAND ${CMAKE_COMMAND} -E env ${MESATEE_COMMON_ENVS} RUSTFLAGS=${RUSTFLAGS}
            ${MT_SCRIPT_DIR}/cargo_build_ex.sh -p ${pkg_name}
            --target-dir ${TRUSTED_TARGET_DIR} ${CARGO_BUILD_FLAGS} ${SGX_ENCLAVE_FEATURES} ${MTEE_EXTRA_CARGO_FLAGS}
        COMMAND ${CMAKE_COMMAND} -E env ${TARGET_SGXLIB_ENVS} SGX_COMMON_CFLAGS=${STR_SGX_COMMON_CFLAGS}
            CUR_MODULE_NAME=${_module_name} CUR_MODULE_PATH=${sgx_lib_path} CUR_INSTALL_DIR=${_copy_dir} ${MT_SCRIPT_DIR}/sgx_link_sign.sh
        ${_depends}
        COMMAND cat ${MESATEE_OUT_DIR}/${_module_name}.enclave.meta.txt | python ${MT_SCRIPT_DIR}/gen_enclave_info_toml.py ${_module_name} > ${_enclave_info}
        COMMENT "Building ${_target_name}, enclave info to ${_enclave_info}"
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

    # Hook the convenience targets for SGX modules
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

function(parse_cargo_packages pkg_names)
    set(options)
    set(oneValueArgs CARGO_TOML_PATH PKG_PATHS CATEGORIES)
    set(multiValueArgs)

    cmake_parse_arguments(MTEE "${options}" "${oneValueArgs}"
        "${multiValueArgs}" ${ARGN})

    set(_output)
    set(err)

    execute_process(
        COMMAND python ${PROJECT_SOURCE_DIR}/cmake/scripts/parse_cargo_packages.py
            ${MTEE_CARGO_TOML_PATH} ${PROJECT_SOURCE_DIR}
        OUTPUT_VARIABLE _output
        ERROR_VARIABLE err
    ) 

    if(NOT (err STREQUAL ""))
        message(FATAL_ERROR "failed to load packages: ${err}")
    endif()

    string(REGEX REPLACE "\n" ";" _out_list ${_output})
    list(LENGTH _out_list LLEN)

    if (DEFINED MTEE_CATEGORIES)
        list(GET _out_list 2 _categories)
        string(REPLACE ":" ";" _categories ${_categories})
        set(${MTEE_CATEGORIES} ${_categories} PARENT_SCOPE)
        dbg_message("${MTEE_CATEGORIES}=${_categories}\n")
    endif()

    if (DEFINED MTEE_PKG_PATHS)
        list(GET _out_list 1 _pkg_paths)
        string(REPLACE ":" ";" _pkg_paths ${_pkg_paths})
        set(${MTEE_PKG_PATHS} ${_pkg_paths} PARENT_SCOPE)
        dbg_message("${MTEE_PKG_PATHS}=${_pkg_paths}\n")
    endif()

    # level up the local variable to its parent scope
    list(GET _out_list 0 _pkg_names)
    string(REPLACE ":" ";" _pkg_names ${_pkg_names})
    set(${pkg_names} ${_pkg_names} PARENT_SCOPE)
    dbg_message("${pkg_names}=${_pkg_names}\n")
endfunction()
