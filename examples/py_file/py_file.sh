#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../release/example
BIN=./py_file
cd $BIN_DIR

PY_SCRIPT=$SCRIPT_DIR/payload.py
RESULT_PATH=/tmp/py_result.txt
GROUND_TRUTH=$SCRIPT_DIR/py_result.txt

# check ports
for port in 5554 5555 3444 6016 5065 5066; do
    if ! lsof -i :$port > /dev/null; then
        echo "[-] port $port is not open"
        echo "[-] please run service.sh start|restart to launch services"
        exit 1
    fi
done

$BIN $PY_SCRIPT > $RESULT_PATH # result is in Python marshal format
cmp $RESULT_PATH $GROUND_TRUTH && echo "Python Execution Successful" || exit 1
