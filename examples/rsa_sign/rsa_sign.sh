#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../release/example
BIN=./rsa_sign
cd $BIN_DIR

KEY_PATH=$SCRIPT_DIR/key.der
KEY_ID_PATH=./key_id.txt
DATA_PATH=$SCRIPT_DIR/data.txt
SIG_PATH=/tmp/out.sig
GROUND_TRUTH=$SCRIPT_DIR/data.sig

# check ports
for port in 5554 5555 3444 6016 5065 5066; do
    if ! lsof -i :$port > /dev/null; then
        echo "[-] port $port is not open"
        echo "[-] please run service.sh start|restart to launch services"
        exit 1
    fi
done

$BIN upload_key $KEY_PATH $KEY_ID_PATH
KEY_ID=`cat $KEY_ID_PATH`
$BIN sign $KEY_ID $DATA_PATH $SIG_PATH

cmp --silent $SIG_PATH $GROUND_TRUTH && echo "RSA Sign Successful" || exit 1
