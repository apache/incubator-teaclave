#!/bin/bash

# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

SGX_DEV_SEL="none"
AESM_SEL="none"

function sgx_dev_detect() {
    local ISGX_DEV=/dev/isgx
    local ISGX_DEV_EXIST=false
    if [ -c "$ISGX_DEV" ]; then
        echo "$ISGX_DEV device detected."
        ISGX_DEV_EXIST=true
    fi

    local ENCL_DEV=/dev/sgx/enclave
    local ENCL_DEV_EXIST=false
    if [ -L "$ENCL_DEV" ] && [ -c $(realpath $ENCL_DEV) ]; then
        echo "$ENCL_DEV device detected."
        ENCL_DEV_EXIST=true
    fi

    local PROV_DEV=/dev/sgx/provision
    local PROV_DEV_EXIST=false
    if [ -L "$PROV_DEV" ] && [ -c $(realpath $PROV_DEV) ]; then
        echo "$PROV_DEV device detected."
        PROV_DEV_EXIST=true
    fi

    if ($ISGX_DEV_EXIST && $ENCL_DEV_EXIST && $PROV_DEV_EXIST); then
        PS3='Please enter your choice: '
        options=("ISGX device" "DCAP device" "Quit")
        select opt in "${options[@]}"
        do
            case $opt in
                "ISGX device")
                    echo "you chose $opt"
                    SGX_DEV_SEL="isgx"
                    break
                    ;;
                "DCAP device")
                    echo "you chose $opt"
                    SGX_DEV_SEL="dcap"
                    break
                    ;;
                "Quit")
                    exit 1
                    ;;
                *) echo "invalid option $REPLY" ;;
            esac
        done
    else
        if $ISGX_DEV_EXIST; then
            SGX_DEV_SEL="isgx"
        fi
        if ($ENCL_DEV_EXIST && $PROV_DEV_EXIST); then
            SGX_DEV_SEL="dcap"
        fi
    fi
}

function aesm_detect() {
    local AESM_SOCK=/var/run/aesmd/aesm.socket
    local AESM_SOCK_EXIST=false
    if [ -S "$AESM_SOCK" ]; then
        echo "$AESM_SOCK socket detected."
        AESM_SOCK_EXIST=true
    fi

    local AESM_VOL=aesmd-socket
    local AESM_VOL_EXIST=false
    if docker volume inspect $AESM_VOL 2>&1 > /dev/null ; then
        echo "$AESM_VOL volume detected."
        AESM_VOL_EXIST=true
    fi

    if ($AESM_SOCK_EXIST && $AESM_VOL_EXIST); then
        PS3='Please enter your choice: '
        options=("$AESM_SOCK socket" "$AESM_VOL volume" "Quit")
        select opt in "${options[@]}"
        do
            case $opt in
                "$AESM_SOCK socket")
                    echo "you chose $opt"
                    AESM_SEL="sock"
                    break
                    ;;
                "$AESM_VOL volume")
                    echo "you chose $opt"
                    AESM_SEL="vol"
                    break
                    ;;
                "Quit")
                    exit 1
                    ;;
                *) echo "invalid option $REPLY" ;;
            esac
        done
    else
        if $AESM_SOCK_EXIST; then
            AESM_SEL="sock"
        fi
        if $AESM_VOL_EXIST; then
            AESM_SEL="vol"
        fi
    fi
}

function usage {
    echo "Usage: $(basename $0) [-hdbm:]" 2>&1
    echo '   -h           shows usage'
    echo '   -m           run mode (default: sgx)'
    echo '   -d           detached mode'
    echo '   -b           build or rebuild services'
    echo 'Available run modes: sim, sgx'
    exit 1
}

RUN_MODE="sgx"
DETACH_ARG=""
optstring="hdbm:"
while getopts ${optstring} arg; do
    case ${arg} in
        h)
            echo "showing usage!"
            usage
            ;;
        d)
            DETACH_ARG="-d"
            ;;
        b)
            BUILD_ARG="--build"
            ;;
        m)
            RUN_MODE=$OPTARG
            ;;
    esac
done

shift $((OPTIND-1))

case $RUN_MODE in
    "sgx")
        sgx_dev_detect
        aesm_detect
        ;;
    "sim")
        ;;
    *)
        echo "The specified run mode: $RUN_MODE is not recognized."
        usage
        ;;
esac

OV_PREFIX="docker-compose-"
OV_SUFFIX=".override.yml"
SGX_DEV_OV_FILE=""
AESM_OV_FILE=""
case $SGX_DEV_SEL in
    "isgx")
        SGX_DEV_OV_FILE="isgx-dev"
        ;;
    "dcap")
        SGX_DEV_OV_FILE="dcap-dev"
        ;;
    "none")
        ;;
    *)
        echo "Invalid SGX device."
        exit 2
        ;;
esac
SGX_DEV_OV_FILE="${OV_PREFIX}${SGX_DEV_OV_FILE}${OV_SUFFIX}"

case $AESM_SEL in
    "sock")
        AESM_OV_FILE="aesm-socket"
        ;;
    "vol")
        AESM_OV_FILE="aesm-vol"
        ;;
    "none")
        ;;
    *)
        echo "Invalid AESM service."
        exit 2
        ;;
esac
AESM_OV_FILE="${OV_PREFIX}${AESM_OV_FILE}${OV_SUFFIX}"

DOCKER_COMPOSE_FILE="docker-compose-ubuntu-1804.yml"
DC_ARGS=""
if [ "$RUN_MODE" == "sgx" ]; then
    if [ "$SGX_DEV_SEL" == "none" ]; then
        echo "Cannot find a valid sgx device."
        exit 3
    fi
    if [ "$AESM_SEL" == "none" ]; then
        echo "Cannot find a valid aesm service."
        exit 6
    fi
    DC_ARGS="-f $DOCKER_COMPOSE_FILE -f $SGX_DEV_OV_FILE -f $AESM_OV_FILE"
else
    DC_ARGS="-f $DOCKER_COMPOSE_FILE"
fi

echo COMMAND: docker-compose ${DC_ARGS} up ${DETACH_ARG} ${BUILD_ARG}
docker-compose ${DC_ARGS} up ${DETACH_ARG} ${BUILD_ARG}
