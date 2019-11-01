#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./kmeans
cd $BIN_DIR

K_NUM=3
FEATURES_NUM=2
TEST_DATA_PATH=$SCRIPT_DIR/test_data.txt
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

# Since k-means randomly select initial points, the classification result is not
# stable. Uncomment this line if you need to assert the result.
# assert_eq "`$BIN $K_NUM $FEATURES_NUM $TEST_DATA_PATH | grep -Eo '[0-9]' | sort | uniq -c | sort -nr | awk '{print $1}' 2>&1`" "`cat $EXPECTED_RESULT`"
