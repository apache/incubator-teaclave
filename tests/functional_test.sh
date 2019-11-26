#!/bin/bash

trap "pkill -2 -P $$; wait" SIGINT SIGTERM EXIT

cd ../release/services

# prepare test data
cp -r ../../tests/integration_test/test_data ./

# check port
if lsof -i :6016; then
    echo "[-] port 6016 is in use"
    exit 1
fi
if lsof -i :5066; then
    echo "[-] port 5066 is in use"
    exit 1
fi
if lsof -i :5077; then
    echo "[-] port 5066 is in use"
    exit 1
fi
if lsof -i :5065; then
    echo "[-] port 5065 is in use"
    exit 1
fi
if lsof -i :5554; then
    echo "[-] port 5554 is in use"
    exit 1
fi
if lsof -i :5555; then
    echo "[-] port 5555 is in use"
    exit 1
fi
if lsof -i :3444; then
    echo "[-] port 3444 is in use"
    exit 1
fi

# run enclave modules in the background
./kms 2>&1 | tee kms.log &
./tdfs 2>&1 | tee tdfs.log &
./tms 2>&1 | tee tms.log &
./fns 2>&1 | tee fns.log &
./acs 2>&1 | tee acs.log &

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

wait_service kms 6016 30
wait_service tdfs 5066 30
wait_service tdfs 5065 30
wait_service tms 5554 30
wait_service tms 5555 30
wait_service fns 3444 30
wait_service acs 5077 30

./functional_test 2>&1 | tee functional_test.log
exit ${PIPESTATUS[0]}
