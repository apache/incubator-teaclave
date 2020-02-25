#!/bin/bash
set -e
REQUIRED_ENVS=("MESATEE_PROJECT_ROOT" "MESATEE_BUILD_ROOT" "MESATEE_SYMLINKS")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

# Tell rustc to remap absolute src paths to make enclaves' signature more reproducible
exec rustc "$@" --remap-path-prefix=${HOME}/.cargo=${MESATEE_SYMLINKS}/cargo_home --remap-path-prefix=${MESATEE_PROJECT_ROOT}=${MESATEE_SYMLINKS}/mesatee_src --remap-path-prefix=${MESATEE_BUILD_ROOT}=${MESATEE_SYMLINKS}/mesatee_build
