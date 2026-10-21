#!/bin/sh
# build.sh — build the reference gate for the same target as the live
# contract (wasm32-unknown-unknown, freestanding, zero-libc) and run the
# host test. The deployed contract (code hash 6N7d4cDC…) is compiled
# exactly this way from the ZXV kernel.
#
# Usage:  ./build.sh          # host test + wasm build (if clang present)
#
# Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
# SPDX-License-Identifier: Apache-2.0
set -e
cd "$(dirname "$0")"

echo "== host test (the 66-check gate verification) =="
cc -O2 -Wall -Wextra -o test_lpres_gate test_lpres_gate.c
./test_lpres_gate

CLANG="${CLANG:-clang}"
if command -v "$CLANG" >/dev/null 2>&1; then
    echo "== wasm32 build (the deployed contract's target) =="
    "$CLANG" --target=wasm32-unknown-unknown -O2 -ffreestanding -nostdlib \
        -fno-stack-protector -mno-bulk-memory -mno-mutable-globals \
        -Wl,--no-entry -Wl,--export=lpres_join_op -Wl,--export=lpres_kjoin \
        -Wl,--export=lpres_conjoin -Wl,--export=lpres_disjoin \
        -Wl,--export=lpres_is_settleable \
        -Wl,--strip-all -Wl,--stack-first -Wl,-z,stack-size=8192 \
        -Wl,--allow-undefined -o lpres_gate.wasm lpres_gate.c
    ls -la lpres_gate.wasm
    echo "== done: host test PASS + wasm artifact built =="
else
    echo "== clang not found: host test PASS, wasm build skipped =="
fi