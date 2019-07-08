#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
BIN_DIR=$SCRIPT_DIR/../../bin
BIN=./private_join_and_compute

cd $BIN_DIR

USER1=Bank_A
TOKEN1=token1
USER2=Bank_B
TOKEN2=token2
USER3=Bank_C
TOKEN3=token3
USER4=Bank_D
TOKEN4=token4

DATA1=$SCRIPT_DIR/four_party_data/bank_a.txt
DATA2=$SCRIPT_DIR/four_party_data/bank_b.txt
DATA3=$SCRIPT_DIR/four_party_data/bank_c.txt
DATA4=$SCRIPT_DIR/four_party_data/bank_d.txt


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

$BIN create_task $USER1 $TOKEN1 $DATA1 "$USER2|$USER3|$USER4" 2>&1 | tee create.log
TASK_ID=`cat create.log | grep "Task_id:" | awk -F': ' '{print $2}' | sed 's/\n//'`

time `$BIN approve_task $USER2 $TOKEN2 $TASK_ID $USER1 $DATA2 > /dev/null`

time `$BIN approve_task $USER3 $TOKEN3 $TASK_ID $USER1 $DATA3 > /dev/null`

time `$BIN approve_task $USER4 $TOKEN4 $TASK_ID $USER1 $DATA4 > /dev/null`

time `$BIN launch_task $USER2 $TOKEN2 $TASK_ID > /dev/null`

time `$BIN get_result $USER1 $TOKEN1 $TASK_ID > /dev/null `
time `$BIN get_result $USER2 $TOKEN2 $TASK_ID > /dev/null `
time `$BIN get_result $USER3 $TOKEN3 $TASK_ID > /dev/null `
time `$BIN get_result $USER4 $TOKEN4 $TASK_ID > /dev/null `
