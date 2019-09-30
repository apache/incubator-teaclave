#!/bin/bash
#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./gbdt
cd $BIN_DIR


#Train
TRAIN_ACTION=train
TRAIN_FEATURE_SIZE=4
TRAIN_MAX_DEPTH=4
TRAIN_ITERATIONS=100
TRAIN_SHRINKAGE=0.1
TRAIN_FEATURE_SAMPLE_RATIO=1.0
TRAIN_DATA_SAMPLE_RATIO=1.0
TRAIN_MIN_LEAF_SIZE=1
TRAIN_LOSS=LAD
TRAIN_TRAINING_OPTIMIZATION_LEVEL=2
TRAIN_DATA_PATH=$SCRIPT_DIR/train.txt
TRAIN_MODEL_FILE_ID_SAVING_PATH=$SCRIPT_DIR/gbdt.model


#PREDICT
PREDICT_ACTION=predict
EXPECTED_RESULT=$SCRIPT_DIR/expected_result.txt
MODEL_DATA_PATH=$SCRIPT_DIR/gbdt.model
TEST_DATA_PATH=$SCRIPT_DIR/test.txt
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

$BIN $TRAIN_ACTION $TRAIN_FEATURE_SIZE $TRAIN_MAX_DEPTH $TRAIN_ITERATIONS $TRAIN_SHRINKAGE $TRAIN_FEATURE_SAMPLE_RATIO $TRAIN_DATA_SAMPLE_RATIO $TRAIN_MIN_LEAF_SIZE $TRAIN_LOSS $TRAIN_TRAINING_OPTIMIZATION_LEVEL $TRAIN_DATA_PATH $TRAIN_MODEL_FILE_ID_SAVING_PATH

MODEL_FILE_ID=$(cat $TRAIN_MODEL_FILE_ID_SAVING_PATH)
assert_eq "`$BIN $PREDICT_ACTION $TEST_DATA_PATH $MODEL_FILE_ID $PREDICT_RESULT_PATH | tail -n +2 2>&1`" "`cat $EXPECTED_RESULT`"