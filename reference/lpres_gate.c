/*
 * lpres_gate.c — the Paraconsistent Logic Gate reference implementation.
 *
 * This is the exact four-valued gate the NEP specifies, extracted from
 * the ZXV kernel (kernel/src/lpres/lpres.c) for standalone reference.
 * It is the implementation the live NEAR mainnet contract runs:
 *   account 6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8
 *   code hash 6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU
 *
 * The four-valued model:
 *   0x00 NEITHER — unobserved / null
 *   0x01 TRUE    — verified assertion
 *   0x02 FALSE   — refuted assertion
 *   0x03 BOTH    — contradictory / disputed evidence
 *
 * Settlement invariant: only TRUE (0x01) authorizes an action. BOTH
 * and NEITHER never authorize. A contradiction is recorded, never
 * resolved by picking a side.
 *
 * Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdint.h>
#include <stdbool.h>

/* the four foundation states (the NEP's 0x00-0x03) */
typedef enum {
    LPRES_STATE_NEITHER = 0,
    LPRES_STATE_TRUE    = 1,
    LPRES_STATE_FALSE   = 2,
    LPRES_STATE_BOTH    = 3
} lpres_state_t;

/* the chain rank: FALSE(2) < NEITHER(0) < BOTH(3) < TRUE(1) on the
 * project's ordering — conjoin = meet (min), disjoin = join (max) */
static const uint8_t lpres_rank[4] = {0, 3, 1, 2};

/* knowledge join: agreement keeps the value; silence (NEITHER) adds
 * nothing; TRUE from one source + FALSE from another = BOTH. The
 * contradiction is retained. (a | b) & 3 is the entire table. */
lpres_state_t lpres_kjoin(lpres_state_t a, lpres_state_t b)
{
    return (lpres_state_t) (((unsigned) a | (unsigned) b) & 3u);
}

/* conjoin (meet): min on the rank. FALSE dominates, TRUE is identity,
 * NEITHER absorbs BOTH. No mix of non-TRUE values can yield TRUE. */
lpres_state_t lpres_conjoin(lpres_state_t a, lpres_state_t b)
{
    if ((unsigned) a >= 4u || (unsigned) b >= 4u)
        return LPRES_STATE_NEITHER;   /* an unknown phase is never authority */
    return lpres_rank[a] <= lpres_rank[b] ? a : b;
}

/* disjoin (join): max on the rank. TRUE dominates, FALSE is identity,
 * BOTH absorbs NEITHER. */
lpres_state_t lpres_disjoin(lpres_state_t a, lpres_state_t b)
{
    if ((unsigned) a >= 4u || (unsigned) b >= 4u)
        return LPRES_STATE_NEITHER;
    return lpres_rank[a] >= lpres_rank[b] ? a : b;
}

/* the settlement invariant: only TRUE authorizes an action. BOTH,
 * FALSE, and NEITHER all block authority. */
bool lpres_is_settleable(lpres_state_t s)
{
    return s == LPRES_STATE_TRUE;
}

/* convenience: dispatch by the NEP's op code (0 conjoin, 1 disjoin,
 * 2 kjoin). */
lpres_state_t lpres_join_op(uint8_t op, lpres_state_t a, lpres_state_t b)
{
    switch (op) {
        case 0: return lpres_conjoin(a, b);
        case 1: return lpres_disjoin(a, b);
        case 2: return lpres_kjoin(a, b);
        default: return LPRES_STATE_NEITHER;
    }
}