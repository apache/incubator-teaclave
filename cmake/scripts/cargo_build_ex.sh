#!/bin/bash
# extended cargo build script

# if MT_RUSTC_WRAPPER is not empty, use it as rustc
RUSTC="${MT_RUSTC_WRAPPER:-${RUSTC}}"

if [ ! -z "${MUTE_CARGO}" ]; then
    RUSTC="${RUSTC}" cargo build "$@" >/dev/null 2>&1
else
    RUSTC="${RUSTC}" cargo build "$@"
fi

# if CLP is set, run cargo clippy after cargo build
# cannot use MT_RUSTC_WRAPPER for cargo clippy
if [ ! -z "$CLP" ]; then
    cargo clippy "$@" -- -D warnings
fi
