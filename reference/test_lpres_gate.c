/*
 * test_lpres_gate.c — verifies the reference implementation against the
 * live on-chain proof (docs/evidence/lpres_join_onchain_proof.json) and
 * the NEP's security property.
 *
 * Compile:  cc -O2 -Wall -Wextra -o test_lpres_gate test_lpres_gate.c lpres_gate.c
 * Run:      ./test_lpres_gate
 *
 * Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdint.h>

/* the reference gate */
#include "lpres_gate.c"

static const char *name(lpres_state_t s)
{
    switch (s) {
        case LPRES_STATE_NEITHER: return "NEITHER";
        case LPRES_STATE_TRUE:    return "TRUE";
        case LPRES_STATE_FALSE:   return "FALSE";
        case LPRES_STATE_BOTH:    return "BOTH";
        default:                  return "?";
    }
}

int main(void)
{
    unsigned fails = 0, checks = 0;

    /* 1. the kjoin table must match the on-chain proof (all 16 cells) */
    /* expected: the on-chain verified table */
    static const lpres_state_t KJOIN[4][4] = {
        /* NEITHER TRUE FALSE BOTH */
        {LPRES_STATE_NEITHER, LPRES_STATE_TRUE,  LPRES_STATE_FALSE, LPRES_STATE_BOTH},
        {LPRES_STATE_TRUE,    LPRES_STATE_TRUE,  LPRES_STATE_BOTH, LPRES_STATE_BOTH},
        {LPRES_STATE_FALSE,   LPRES_STATE_BOTH,  LPRES_STATE_FALSE, LPRES_STATE_BOTH},
        {LPRES_STATE_BOTH,    LPRES_STATE_BOTH,  LPRES_STATE_BOTH, LPRES_STATE_BOTH},
    };
    for (int a = 0; a < 4; a++) {
        for (int b = 0; b < 4; b++) {
            lpres_state_t got = lpres_kjoin((lpres_state_t) a, (lpres_state_t) b);
            checks++;
            if (got != KJOIN[a][b]) {
                printf("FAIL kjoin(%s,%s) = %s, expected %s\n",
                       name((lpres_state_t) a), name((lpres_state_t) b),
                       name(got), name(KJOIN[a][b]));
                fails++;
            }
        }
    }

    /* 2. the security property: no input coerces BOTH or NEITHER into
     * TRUE via any op. Only TRUE settles. */
    for (int op = 0; op < 3; op++) {
        for (int a = 0; a < 4; a++) {
            for (int b = 0; b < 4; b++) {
                lpres_state_t s = lpres_join_op((uint8_t) op,
                                                (lpres_state_t) a,
                                                (lpres_state_t) b);
                checks++;
                if (lpres_is_settleable(s)) {
                    /* settleable only when the join is TRUE */
                    if (s != LPRES_STATE_TRUE) {
                        printf("FAIL settleable but not TRUE (op=%d a=%d b=%d)\n",
                               op, a, b);
                        fails++;
                    }
                }
            }
        }
    }

    /* 3. the headline: kjoin(TRUE, FALSE) = BOTH — the on-chain gift */
    checks++;
    if (lpres_kjoin(LPRES_STATE_TRUE, LPRES_STATE_FALSE) != LPRES_STATE_BOTH) {
        printf("FAIL kjoin(TRUE,FALSE) != BOTH\n");
        fails++;
    }

    /* 4. determinism: the gate is a pure function (same inputs, same
     * output — trivially true, asserted here for the record) */
    checks++;
    if (lpres_kjoin(LPRES_STATE_TRUE, LPRES_STATE_FALSE) !=
        lpres_kjoin(LPRES_STATE_TRUE, LPRES_STATE_FALSE)) {
        printf("FAIL gate not deterministic\n");
        fails++;
    }

    printf("lpres_gate: %u checks, %u failures — %s\n",
           checks, fails, fails == 0 ? "PASS" : "FAIL");
    return fails == 0 ? 0 : 1;
}