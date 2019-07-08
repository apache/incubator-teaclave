#!/bin/bash
# extended cargo build script
# if CLP is set, run cargo clippy after cargo build

if [ ! -z "${MUTE_CARGO}" ]; then
    cargo build "$@" >/dev/null 2>&1
else
    cargo build "$@"
fi

if [ ! -z "$CLP" ]; then
    cargo clippy "$@" -- -D warnings
fi
