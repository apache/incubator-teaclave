#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../release/example
BIN=./dbscan
cd $BIN_DIR

EPS=0.5
MIN_POINTS=2
INPUT_MODEL_DATA_COLUMNS=2
INPUT_PATH=$SCRIPT_DIR/input.txt
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
assert_eq "`$BIN $EPS $MIN_POINTS $INPUT_MODEL_DATA_COLUMNS $INPUT_PATH  | tail -n +1 2>&1`" "`cat $EXPECTED_RESULT`"
