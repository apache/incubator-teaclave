#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../release/example
BIN=./lin_reg
cd $BIN_DIR

INPUT_MODEL_DATA_COLUMNS=13
INPUT_PATH=$SCRIPT_DIR/input.txt
TARGET_PATH=$SCRIPT_DIR/target.txt
TEST_DATA_PATH=$SCRIPT_DIR/test.txt
EXPECTED_RESULT=$SCRIPT_DIR/expected_result.txt

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
assert_eq "`$BIN $INPUT_MODEL_DATA_COLUMNS $INPUT_PATH $TARGET_PATH $TEST_DATA_PATH | tail -n +1 2>&1`" "`cat $EXPECTED_RESULT`"
