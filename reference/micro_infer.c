/* micro_infer.c — minimal on-chain micro-inference (the AIVM wasm path).
 *
 * A BitNet-1.58-style ternary ([-1,0,+1]) lookup-table embedding
 * evaluation, compiled to the SAME freestanding wasm32 target as the
 * kernel. Deterministic by construction (integer-only, no floats):
 * the sandbox output is the inference result, TRUE by construction.
 *
 * Surface:
 *   micro_embed(token, dim) -> activation (integer dot of ternary table)
 *   micro_lookup(token)     -> the ternary row (packed 2-bit)
 *   micro_check(token, dim) -> known-answer vector check (determinism)
 *
 * This is the reference for benchmarking wasmer gas on a NEAR testnet
 * satellite (deploy when funded). Compile:
 *   clang --target=wasm32-unknown-unknown -Oz -ffreestanding -nostdlib \
 *     -fno-stack-protector -mno-bulk-memory -mno-mutable-globals \
 *     -Wl,--no-entry -Wl,--export=micro_embed -Wl,--export=micro_lookup \
 *     -Wl,--export=micro_check -Wl,--strip-all -Wl,--stack-first \
 *     -Wl,-z,stack-size=8192 -Wl,--allow-undefined -o micro_infer.wasm \
 *     micro_infer.c
 *
 * Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>

/* a tiny 8x8 ternary embedding table: packed 2-bit cells
 * (0 = -1, 1 = 0, 2 = +1), 16 bytes per row -> 128 bytes total */
static const uint8_t EMBED[64] = {
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
    0x0E, 0x1F, 0x2A, 0x0B, 0x1C, 0x2D, 0x0E, 0x1F,
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
    0x0E, 0x1F, 0x2A, 0x0B, 0x1C, 0x2D, 0x0E, 0x1F,
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
};

/* decode a packed 2-bit ternary cell: 0 -> -1, 1 -> 0, 2 -> +1 */
static inline int8_t cell(uint8_t packed, uint32_t i)
{
    uint8_t c = (packed >> (2 * (i & 3))) & 0x3;
    return (int8_t) (c == 0 ? -1 : c == 2 ? 1 : 0);
}

/* micro_embed(token, dim) — the integer dot of the token's ternary row
 * against itself, the deterministic activation for dim in [1..8]. */
int32_t micro_embed(uint32_t token, uint32_t dim)
{
    if (dim == 0 || dim > 8)
        return 0;
    if (token >= 8)
        token %= 8;
    const uint8_t *row = &EMBED[token * 8];
    int32_t acc = 0;
    uint32_t i;
    for (i = 0; i < dim; i++) {
        int8_t v = cell(row[i / 4], i);
        acc += (int32_t) v * (int32_t) v; /* ternary self-dot: 0 or 1 */
    }
    return acc;
}

/* micro_lookup(token) — the packed ternary row (8 bytes), for the KAT */
uint64_t micro_lookup(uint32_t token)
{
    if (token >= 8)
        token %= 8;
    uint64_t out = 0;
    uint32_t i;
    const uint8_t *row = &EMBED[token * 8];
    for (i = 0; i < 8; i++)
        out = (out << 8) | row[i];
    return out;
}

/* micro_check(token, dim) — the known-answer vector: returns 1 when
 * micro_embed matches the precomputed expected value (determinism). */
int32_t micro_check(uint32_t token, uint32_t dim)
{
    /* the known-answer vectors, computed from the table (ground truth) */
    static const int32_t EXPECTED[8][9] = {
        {0, 1, 2, 2, 3, 3, 4, 5, 6},
        {0, 1, 1, 2, 3, 3, 3, 4, 5},
        {0, 1, 1, 2, 3, 3, 3, 3, 4},
        {0, 1, 2, 2, 3, 3, 4, 5, 6},
        {0, 1, 1, 2, 3, 3, 3, 4, 5},
        {0, 1, 1, 2, 3, 3, 3, 3, 4},
        {0, 1, 2, 2, 3, 3, 4, 5, 6},
        {0, 1, 1, 2, 3, 3, 3, 4, 5},
    };
    if (token >= 8)
        token %= 8;
    if (dim == 0 || dim > 8)
        return 0;
    return micro_embed(token, dim) == EXPECTED[token][dim] ? 1 : 0;
}