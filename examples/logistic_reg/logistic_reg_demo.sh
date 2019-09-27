#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./logistic_reg
cd $BIN_DIR

TRAIN_ALG_ALPHA=0.3
TRAIN_ALG_ITERS=100
TRAIN_ACTION=train
PREDICT_ACTION=predict
TRAIN_DATA_PATH=$SCRIPT_DIR/train.txt
TARGET_DATA_PATH=$SCRIPT_DIR/target.txt
TEST_DATA_PATH=$SCRIPT_DIR/test.txt
EXPECTED_RESULT=$SCRIPT_DIR/expected_result.txt
TRAIN_MODEL_FILE_ID_SAVING_PATH=$SCRIPT_DIR/test.model
PREDICT_RESULT_PATH=$SCRIPT_DIR/predict.result

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

$BIN $TRAIN_ACTION $TRAIN_ALG_ALPHA $TRAIN_ALG_ITERS $TRAIN_DATA_PATH $TARGET_DATA_PATH $TRAIN_MODEL_FILE_ID_SAVING_PATH 2>&1
MODEL_FILE_ID=$(cat $TRAIN_MODEL_FILE_ID_SAVING_PATH)
$BIN $PREDICT_ACTION $MODEL_FILE_ID $TEST_DATA_PATH $PREDICT_RESULT_PATH 2>&1
