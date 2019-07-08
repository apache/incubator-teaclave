#!/bin/bash
#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./ml_predict
cd $BIN_DIR

EXPECTED_RESULT=$SCRIPT_DIR/expected_result.txt
MODEL_DATA_PATH=$SCRIPT_DIR/gbdt.model
TEST_DATA_PATH=$SCRIPT_DIR/test.txt

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
assert_eq "`$BIN $TEST_DATA_PATH $MODEL_DATA_PATH | tail -n +2 2>&1`" "`cat $EXPECTED_RESULT`"