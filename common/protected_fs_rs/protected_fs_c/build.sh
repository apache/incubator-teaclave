#!/bin/bash

set -e 

display_usage() { 
	printf "Usage:\n  %s --build <dir> --mode {sgx|non_sgx} \n"  "$(basename "$0")"
} 

OPTS=`getopt -o b:m:t:d --long build_dir:,mode:,target:,build_type: -n 'parse-options' -- "$@"`

if [ $? != 0 ]
then 
    display_usage
    exit 1
fi

SOURCE_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

while true; do
  case "$1" in
    -b | --build_dir ) BUILD_DIR="$2"; shift; shift ;;
    -m | --mode ) 
        case "$2" in 
            sgx) MODE="-DNON_SGX_PROTECTED_FS=OFF";;
            non_sgx) MODE="-DNON_SGX_PROTECTED_FS=ON";;
            *) echo "Invalid mode provided!";;
        esac
        shift; shift ;;
    -t | --target ) 
        case "$2" in 
            aarch64-apple-ios) TARGET_FLAGS="-G Xcode -DCMAKE_TOOLCHAIN_FILE=${SOURCE_DIR}/ios.toolchain.cmake -DPLATFORM=OS64";;
            x86_64-apple-ios) TARGET_FLAGS="-G Xcode -DCMAKE_TOOLCHAIN_FILE=${SOURCE_DIR}/ios.toolchain.cmake -DPLATFORM=SIMULATOR64";;
            *) TARGET_FLAGS="";;
        esac
        shift; shift ;;
    -d | --build_type) 
        case "$2" in 
            Release) BUILD_TYPE="--config Release";;
            Debug) BUILD_TYPE="--config Debug";;
            *) echo "Invalid build_type provided!";;
        esac
        shift; shift ;;
    -- ) shift; break ;;
    * ) break ;;
  esac
done

if [ -z "$BUILD_DIR" ] || [ -z "$MODE" ] || [ -z "$BUILD_TYPE" ]
then 
    display_usage
    exit 1
fi


mkdir -p "${BUILD_DIR}"
cd "${BUILD_DIR}"
cmake ${TARGET_FLAGS} ${MODE} ${BUILD_TYPE} "${SOURCE_DIR}"

# We need to force build with -j1 here.
MAKEFLAGS=-j1 cmake --build . ${BUILD_TYPE}
