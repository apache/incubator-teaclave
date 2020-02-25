#!/bin/bash
set -e
REQUIRED_ENVS=("TEACLAVE_PROJECT_ROOT" "TEACLAVE_BUILD_ROOT" "TEACLAVE_SYMLINKS" "CMAKE_C_COMPILER")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

# Tell gcc/clang to remap absolute src paths to make enclaves' signature more reproducible
exec "${CMAKE_C_COMPILER}" "$@" -fdebug-prefix-map=${TEACLAVE_PROJECT_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_src -fdebug-prefix-map=${TEACLAVE_BUILD_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_build
