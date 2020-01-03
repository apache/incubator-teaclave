#!/bin/bash
trap "pkill -2 -P $$; wait" SIGINT SIGTERM EXIT

echo "[+] Running module test: protected_fs_rs ..."
cd ../teaclave_common/protected_fs_rs
cargo test
