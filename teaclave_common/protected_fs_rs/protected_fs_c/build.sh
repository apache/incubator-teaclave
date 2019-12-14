#!/bin/bash

set -e 

display_usage() { 
	printf "Usage:\n  %s --build <dir> --mode {sgx|non_sgx} \n"  "$(basename "$0")"
} 

OPTS=`getopt -o b:m: --long build:,mode: -n 'parse-options' -- "$@"`

if [ $? != 0 ]
then 
    display_usage
    exit 1
fi

while true; do
  case "$1" in
    -b | --build ) BUILD_DIR="$2"; shift; shift ;;
    -m | --mode) 
        case "$2" in 
            sgx) MODE="-DNON_SGX_PROTECTED_FS=OFF";;
            non_sgx) MODE="-DNON_SGX_PROTECTED_FS=ON";;
            *) echo "Invalid mode provided!";;
        esac
        shift; shift ;;
    -- ) shift; break ;;
    * ) break ;;
  esac
done

if [ -z "$BUILD_DIR" ] || [ -z "$MODE" ]
then 
    display_usage
    exit 1
fi

SOURCE_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

mkdir -p "${BUILD_DIR}"
cd "${BUILD_DIR}"
cmake "${MODE}" "${SOURCE_DIR}"
make -j1

# Final libraries will be installed to $BUILD_DIR/target
