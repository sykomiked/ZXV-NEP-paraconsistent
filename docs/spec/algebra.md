# Mathematical & Algebraic Primitives — `lpres_join`

**Formal specification of the four-valued paraconsistent logic gate.**

This document is the algebraic foundation reviewers will ask for: the
lattice, the laws, and the non-coercion invariant — stated precisely,
including which laws hold for which operator.

---

## 1. The carrier set

Let `S = {NEITHER, TRUE, FALSE, BOTH}` with codes:

| Code | State | Semantics |
|---|---|---|
| `0x00` | `NEITHER` | unobserved / null / unattested |
| `0x01` | `TRUE` | verified assertion |
| `0x02` | `FALSE` | refuted assertion |
| `0x03` | `BOTH` | contradictory / disputed evidence |

## 2. The three operators

### 2.1 `kjoin` (knowledge join) — op 2

```
kjoin(a, b) = (a | b) & 0x03
```

Bitwise OR masked to the low two bits. This is the **evidence
accumulation** operator: every observed value contributes its bits; the
mask keeps the result in `S`.

### 2.2 `conjoin` (meet) — op 0

```
conjoin(a, b) = rank⁻¹( min(rank(a), rank(b)) )
```

where `rank` is the project's chain ordering (see §3).

### 2.3 `disjoin` (join) — op 1

```
disjoin(a, b) = rank⁻¹( max(rank(a), rank(b)) )
```

## 3. The chain rank (the ordering)

The meet/join operators are defined on a **chain**, not a lattice. The
rank map (the project's deliberate deviation from Belnap's FOUR):

| State | Rank | Intuition |
|---|---|---|
| `NEITHER` | 0 | nothing known |
| `FALSE` | 1 | refuted |
| `BOTH` | 2 | disputed |
| `TRUE` | 3 | verified — the only authority |

So: `NEITHER < FALSE < BOTH < TRUE`.

## 4. Algebraic laws

### 4.1 Laws that hold for **all three** operators

| Law | Statement | kjoin | conjoin | disjoin |
|---|---|---|---|---|
| Commutativity | `f(a,b) = f(b,a)` | ✓ (OR) | ✓ (min) | ✓ (max) |
| Associativity | `f(f(a,b),c) = f(a,f(b,c))` | ✓ (OR) | ✓ (min) | ✓ (max) |
| Idempotence | `f(a,a) = a` | ✓ (`(a\|a)&3=a`) | ✓ (min) | ✓ (max) |
| Absorption of `NEITHER` (identity for kjoin) | `kjoin(a, NEITHER) = a` | ✓ | — | — |

Proof of associativity for kjoin:

```
kjoin(kjoin(a,b),c) = (((a|b)&3) | c) & 3
                    = ((a|b)|c) & 3      since ((a|b)&3)|c keeps only low 2 bits after &3
                    = (a|(b|c)) & 3
                    = kjoin(a, kjoin(b,c))   ∎
```

Commutativity for kjoin follows from OR's commutativity; idempotence
from `(a|a)&3 = a&3 = a` for all `a ∈ S`.

### 4.2 The Absorption Law (contradiction collapse)

```
kjoin(TRUE, FALSE)  = (1|2)&3 = 3 = BOTH
kjoin(BOTH,  TRUE)  = (3|1)&3 = 3 = BOTH
kjoin(BOTH, FALSE)  = (3|2)&3 = 3 = BOTH
kjoin(BOTH, BOTH)   = (3|3)&3 = 3 = BOTH
kjoin(BOTH, a)      = 3        for all a ∈ S
```

**`BOTH` is absorbing under kjoin.** Once a contradiction enters the
accumulation, no further kjoin removes it. Information is never
destroyed by resolution.

### 4.3 The Non-Coercion Invariant (the security theorem)

**Theorem.** For all `a, b ∈ S`, `kjoin(a, b) = TRUE` **iff** `{a, b} ⊆ {TRUE, NEITHER}` and at least one of `a, b` is `TRUE`.

*Proof.* `kjoin(a,b) = (a|b)&3 = 1` requires the low two bits of `a|b`
to be exactly `01` (binary). That is possible only if neither `a` nor
`b` has bit 1 (the `FALSE`/`BOTH` bit) set — i.e. neither is `FALSE`
(2) nor `BOTH` (3) — and at least one has bit 0 set, i.e. at least one
is `TRUE` (1) (or `BOTH`, excluded). The only elements without bit 1
are `NEITHER (0)` and `TRUE (1)`. Hence the condition. ∎

**Corollary (no coercion).** No finite sequence of kjoins can produce
`TRUE` from any input set containing `FALSE` or `BOTH`. In particular,
a `BOTH` (dispute) can never be elevated to `TRUE` (authorization)
without an explicit, cryptographically authenticated override outside
the gate — and the gate itself has no such override.

**Corollary (settlement).** `lpres_is_settleable(kjoin(a,b))` is true
exactly for `(TRUE,TRUE)`, `(TRUE,NEITHER)`, `(NEITHER,TRUE)` — the
three cells in the KAT where the result is `TRUE`.

### 4.4 Conjoin/disjoin laws (the honest statement)

For `conjoin` (meet on the chain): `conjoin(a,b) = TRUE` iff
`a = b = TRUE` (both must be rank 3). `conjoin(BOTH, TRUE) = BOTH` —
a dispute is not coerced.

For `disjoin` (join on the chain): `disjoin(a,b) = TRUE` iff
`max(rank(a),rank(b)) = 3`, i.e. **either** input is `TRUE`.
`disjoin(BOTH, TRUE) = TRUE`. **This is by design**: disjoin is the
existential operator ("some source asserts"), not the settlement
operator. The settlement invariant is defined on `kjoin` (and
`conjoin`), where the non-coercion theorem holds; `disjoin` is never
used to authorize settlement. This asymmetry is intentional and
documented so reviewers cannot mistake the two.

## 5. The complete tables

### kjoin (op 2)

| kjoin | NEITHER | TRUE | FALSE | BOTH |
|---|---|---|---|---|
| **NEITHER** | NEITHER | TRUE | FALSE | BOTH |
| **TRUE** | TRUE | TRUE | BOTH | BOTH |
| **FALSE** | FALSE | BOTH | FALSE | BOTH |
| **BOTH** | BOTH | BOTH | BOTH | BOTH |

### conjoin (op 0, meet)

| conjoin | NEITHER | TRUE | FALSE | BOTH |
|---|---|---|---|---|
| **NEITHER** | NEITHER | NEITHER | NEITHER | NEITHER |
| **TRUE** | NEITHER | TRUE | FALSE | BOTH |
| **FALSE** | NEITHER | FALSE | FALSE | FALSE |
| **BOTH** | NEITHER | BOTH | FALSE | BOTH |

### disjoin (op 1, join)

| disjoin | NEITHER | TRUE | FALSE | BOTH |
|---|---|---|---|---|
| **NEITHER** | NEITHER | TRUE | FALSE | BOTH |
| **TRUE** | TRUE | TRUE | TRUE | TRUE |
| **FALSE** | FALSE | TRUE | FALSE | BOTH |
| **BOTH** | BOTH | TRUE | BOTH | BOTH |

## 6. Consistency with the on-chain proof

All 48 cells above were verified **live on NEAR mainnet** 2026-10-20
via `lpres_join(op, a, b)` on contract
`6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
(code hash `6N7d4cDC…`). The proof file
(`docs/evidence/lpres_join_onchain_proof.json`) records all 48 cells.