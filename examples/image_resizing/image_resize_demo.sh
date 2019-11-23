#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../release/example

BIN=./image_resizing

cd $BIN_DIR

IMAGE_PATH=$SCRIPT_DIR/logo.png
WIDTH=100
HEIGHT=100
FILTER=Nearest
OUTPUT_FORMAT=JPEG
OUTPUT_PATH=./logo.jpg
EXPECTED_RESULT=$SCRIPT_DIR/expected_result.txt

assert_eq() {
  if [ "`echo $1`" != "`echo $2`" ]; then
    echo "Result mismatch:"
    diff <(echo "$1") <(echo $2)
    exit 1
  else
    echo "$1"
  fi
}

# check ports
for port in 5554 5555 3444 6016 5065 5066; do
    if ! lsof -i :$port > /dev/null; then
        echo "[-] port $port is not open"
        echo "[-] please run service.sh start|restart to launch services"
        exit 1
    fi
done

$BIN $IMAGE_PATH $WIDTH $HEIGHT $FILTER $OUTPUT_FORMAT $OUTPUT_PATH 2>&1
assert_eq "`file $OUTPUT_PATH`" "`cat $EXPECTED_RESULT`"
