#!/bin/bash
set -e
REQUIRED_ENVS=("TEACLAVE_PROJECT_ROOT" "TEACLAVE_BUILD_ROOT" "TEACLAVE_SYMLINKS")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

# Tell rustc to remap absolute src paths to make enclaves' signature more reproducible
exec rustc "$@" --remap-path-prefix=${HOME}/.cargo=${TEACLAVE_SYMLINKS}/cargo_home --remap-path-prefix=${TEACLAVE_PROJECT_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_src --remap-path-prefix=${TEACLAVE_BUILD_ROOT}=${TEACLAVE_SYMLINKS}/teaclave_build
