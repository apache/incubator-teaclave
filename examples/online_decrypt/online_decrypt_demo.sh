#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./online_decrypt
cd $BIN_DIR

PLAINTXT=$SCRIPT_DIR/test.txt
CIPHERTXT=./cipher.txt
KEY_PATH=./key.txt
KEY_ID_PATH=./key_id.txt
OUTPUT=./output.txt

# check ports
for port in 5554 5555 3444 6016 5065 5066; do
    if ! lsof -i :$port > /dev/null; then
        echo "[-] port $port is not open"
        echo "[-] please run service.sh start|restart to launch services"
        exit 1
    fi
done

assert_eq() {
  if [ "`echo $1`" != "`echo $2`" ]; then
    echo "Result mismatch:"
    diff <(echo "$1") <(echo $2)
    exit 1
  else
    echo "$1"
  fi
}

$BIN gen_and_upload_key $KEY_PATH $KEY_ID_PATH
$BIN local_encrypt $PLAINTXT $CIPHERTXT $KEY_PATH
KEY_ID=`cat $KEY_ID_PATH`
$BIN online_decrypt $CIPHERTXT $KEY_ID $OUTPUT

assert_eq "`cat $OUTPUT`" "`cat $PLAINTXT`"