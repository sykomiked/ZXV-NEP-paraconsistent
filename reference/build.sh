#!/bin/sh
# build.sh — build the AIVM reference artifacts (gate + micro-inference).
#
# Builds:
#   lpres_gate.wasm     the paraconsistent gate (the NEP's subject)
#   micro_infer.wasm    the on-chain micro-inference engine (AIVM wasm path)
# plus runs the host tests for both.
#
# Usage:  ./build.sh
#
# Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
# SPDX-License-Identifier: Apache-2.0
set -e
cd "$(dirname "$0")"

CLANG="${CLANG:-clang}"
WASM_FLAGS="--target=wasm32-unknown-unknown -Oz -ffreestanding -nostdlib \
    -fno-stack-protector -mno-bulk-memory -mno-mutable-globals \
    -Wl,--no-entry -Wl,--strip-all -Wl,--stack-first -Wl,-z,stack-size=8192 \
    -Wl,--allow-undefined"

echo "== 1. paraconsistent gate: host test =="
cc -O2 -Wall -Wextra -o test_lpres_gate test_lpres_gate.c
./test_lpres_gate

echo "== 2. paraconsistent gate: wasm build =="
if command -v "$CLANG" >/dev/null 2>&1; then
    "$CLANG" $WASM_FLAGS \
        -Wl,--export=lpres_join_op -Wl,--export=lpres_kjoin \
        -Wl,--export=lpres_conjoin -Wl,--export=lpres_disjoin \
        -Wl,--export=lpres_is_settleable \
        -o lpres_gate.wasm lpres_gate.c
    echo "   lpres_gate.wasm: $(wc -c < lpres_gate.wasm) bytes"
fi

echo "== 3. micro-inference engine: wasm build =="
if command -v "$CLANG" >/dev/null 2>&1; then
    "$CLANG" $WASM_FLAGS \
        -Wl,--export=micro_embed -Wl,--export=micro_lookup \
        -Wl,--export=micro_check \
        -o micro_infer.wasm micro_infer.c
    # strip the custom section the NEAR/hook validators reject
    if command -v llvm-objcopy >/dev/null 2>&1; then
        llvm-objcopy --remove-section=target_features micro_infer.wasm micro_infer.wasm
    fi
    echo "   micro_infer.wasm: $(wc -c < micro_infer.wasm) bytes"
fi

echo "== 4. micro-inference KAT (64/64 expected) =="
if command -v node >/dev/null 2>&1; then
    node -e "
      const fs=require('fs');
      WebAssembly.instantiate(fs.readFileSync('micro_infer_clean.wasm')||fs.readFileSync('micro_infer.wasm'),{}).then(r=>{
        const e=r.instance.exports; let p=0,f=0;
        for(let t=0;t<8;t++) for(let d=1;d<=8;d++) e.micro_check(t,d)===1?p++:f++;
        console.log('   KAT:', p+'/64 pass,', f, 'fail');
      });
    "
fi

echo "== done =="