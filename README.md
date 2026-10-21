# ZXV-NEP-paraconsistent

NEAR Enhancement Proposal (NEP) — **Paraconsistent Logic Resolution for
Multi-Agent On-Chain State Machines**.

A standardized four-valued paraconsistent logic resolution matrix
(`lpres_join`) allowing autonomous agents and oracles to ingest
contradictory inputs (`TRUE`, `FALSE`, `BOTH`, `NEITHER`) without
triggering execution panics, while strictly gating value settlement to
verified `TRUE` states.

## Status

**Stage 1-2: Pre-proposal + Draft.** Live proof-of-concept on NEAR
mainnet; formal NEP draft prepared; discussion thread to be opened in
`near/NEPs` (Stage 1) before the draft PR (Stage 2).

## Contents

| Path | What |
|---|---|
| `neps/nep-paraconsistent-gate.md` | the formal NEP document (official template structure) |
| `docs/pre-proposal-discussion.md` | the Stage-1 discussion text (paste into near/NEPs Discussions → Standards) |
| `docs/evidence/lpres_join_onchain_proof.json` | all 16 cells of the kjoin gate, verified live on mainnet |
| `reference/lpres_gate.c` | the standalone reference implementation (exact kernel gate) |
| `reference/test_lpres_gate.c` | the 66-check test: kjoin table == on-chain proof, security property |
| `reference/build.sh` | host test + wasm32 build (the deployed contract's target) |
| `reference/CONTRACT.md` | the live contract surface + kernel source pointers |
| `reference/lpres.rs` | the Rust crate (`LogicState` + `ParaconsistentGate`), 7 tests pass |
| `docs/spec/algebra.md` | the algebraic laws: lattice, absorption, non-coercion theorem |
| `docs/spec/runtime-storage.md` | state layout, gas/memory envelope, monolith-vs-chunking |
| `docs/architecture/subsystems.md` | the kernel→userland OS topology |
| `contracts/` | the 4 satellite contracts (oracle-gateway, ipc-bus, execution-shield, consumer-example) |

## The live proof

- **Contract:** `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
- **Code hash (verified 2026-10-20):** `6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU`
- **On-chain:** `lpres_join(kjoin, TRUE, FALSE)` → `3` (`BOTH`)

Security property (verified on-chain, all 16 cells): **no combination
of inputs can coerce a `BOTH` (dispute) or `NEITHER` (unobserved) state
into `TRUE` (authorization).** Only `TRUE` settles.

## The four-valued model

| Code | State | Meaning |
|---|---|---|
| `0x00` | `NEITHER` | Unobserved / Null |
| `0x01` | `TRUE` | Verified assertion |
| `0x02` | `FALSE` | Refuted assertion |
| `0x03` | `BOTH` | Contradictory / disputed evidence |

`kjoin(TRUE, FALSE) = BOTH` — a contradiction is **recorded, never
resolved by picking a side**; and `BOTH` never authorizes.

## The reference implementation

`reference/` carries the exact gate the NEP specifies, extracted from
the ZXV kernel: `lpres_kjoin = (a|b)&3`, conjoin/disjoin by chain rank,
and `lpres_is_settleable` (only TRUE authorizes). The test verifies
**66 checks against the live on-chain proof** — the kjoin table matches
mainnet cell-for-cell, and no input coerces BOTH/NEITHER into TRUE.

```sh
cd reference && ./build.sh     # host test PASS + wasm32 artifact
```

## License

Apache-2.0. Copyright (c) 2024-2026 Michael Laurence Curzi and
36N9 Genetics, LLC.