
#!/bin/bash

KCOV=kcov
KCOV_OPTS="--verify --exclude-pattern=/.cargo,/glibc,/usr/lib,/usr/include"
KCOV_OUT="./kcov-out/"

export RUSTFLAGS="-C link-dead-code"

TEST_BIN=$(cargo test 2>&1 >/dev/null | awk '/^     Running target\/debug\// { print $2 }')

${KCOV} ${KCOV_OPTS} ${KCOV_OUT} ${TEST_BIN} && xdg-open ${KCOV_OUT}/index.html
