#!/bin/bash

ENABLE_TMS=$1
ENABLE_TDFS=$2
ENABLE_KMS=$3
ENABLE_FNS=$4

trap "pkill -2 -P $$; wait" SIGINT SIGTERM EXIT
cd ../bin

# prepare test data
cp -r ../tests/integration_test/test_data ../bin/

# check port
for port in 5554 5555 3444 6016 5065 5066; do
    if lsof -i :$port; then
        echo "[-] port $port is in use"
        exit 1
    fi
done

wait_service() {
    name=$1
    port=$2
    timeout=$3

    echo "[+] Waiting $name to launch on port $port... "
    timeout $timeout sh -c 'until lsof -i :$0 > /dev/null; do sleep 0.5; done' $port || {
        echo "[-] Timeout, waiting $name on $port"
        exit 1
    }
    echo "[+] $name launched"
}

# run enclave modules in the background

pid_array=()
if [ $ENABLE_KMS -gt 0 ]
then
    ./kms > /dev/null 2>&1 &
    pid=$!
    pid_array+=($pid)
    wait_service kms 6016 30
fi

if [ $ENABLE_TDFS -gt 0 ]
then
    ./tdfs > /dev/null 2>&1 &
    pid=$!
    pid_array+=($pid)
    wait_service tdfs 5066 30
    wait_service tdfs 5065 30
fi

if [ $ENABLE_TMS -gt 0 ]
then
    ./tms > /dev/null 2>&1 &
    pid=$!
    pid_array+=($pid)
    wait_service tms 5554 30
    wait_service tms 5555 30
fi

if [ $ENABLE_FNS -gt 0 ]
then
    ./fns > /dev/null 2>&1 &
    pid=$!
    pid_array+=($pid)
    wait_service fns 3444 30
fi

./functional_test > /dev/null 2>&1
./integration_test > /dev/null 2>&1
RET_STATUS=0
for pid in "${pid_array[@]}"
do
    ps -p $pid > /dev/null
    RET_STATUS=$(($RET_STATUS + $?))
done
echo $RET_STATUS
exit $RET_STATUS
