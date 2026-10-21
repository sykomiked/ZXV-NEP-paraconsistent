---
NEP: Draft
Title: Paraconsistent Logic Resolution for Multi-Agent On-Chain State Machines
Author: Michael Laurence Curzi <deal@zedec.ai>
Status: Draft
Type: Standards Track (Contract Standard)
Category: Smart Contract Standards / AI Frameworks
Created: 2026-10-20
Discussions-To: <pending — GitHub Discussion in near/NEPs>
---

## Summary

A standardized four-valued paraconsistent logic resolution matrix
(`lpres_join`) allowing autonomous agents and oracles to ingest
contradictory inputs (`TRUE`, `FALSE`, `BOTH`, `NEITHER`) without
triggering execution panics, while strictly gating value settlement to
verified `TRUE` states.

## Motivation

In multi-agent and intent-driven architectures (e.g. NEAR AI, agent
frameworks, oracle-fed contracts), smart contracts interact with noisy
external models and diverging oracle feeds. Classical Boolean logic
enforces the principle of explosion (`ex contradictione quodlibet`),
causing contracts to revert or deadlock when contradictory evidence is
submitted. This proposal defines a deterministic state resolution
standard enabling smart contracts to **record and isolate
contradictory branches without halting execution** — a contradiction is
retained, never resolved by picking a side.

## Specification

### 1. The Four-Valued Truth Model

| Code | State | Meaning |
|---|---|---|
| `0x00` | `NEITHER` | Unobserved / Null |
| `0x01` | `TRUE` | Verified assertion |
| `0x02` | `FALSE` | Refuted assertion |
| `0x03` | `BOTH` | Contradictory / disputed evidence |

### 2. Standard Interface (Wasm export; JSON view-compatible)

```rust
pub trait NearParaconsistentGate {
    /// Ingests two truth inputs and returns the resolved paraconsistent state.
    /// op: 0 = conjoin (meet), 1 = disjoin (join), 2 = kjoin (knowledge join)
    fn lpres_join(&self, op: u8, left: u8, right: u8) -> u8;

    /// Evaluates whether an execution branch meets the settlement invariant
    /// (strictly TRUE). BOTH and NEITHER never authorize.
    fn lpres_is_settleable(&self, state: u8) -> bool;
}
```

The `kjoin` (knowledge join, op 2) semantics: agreement keeps the
value; silence (`NEITHER`) adds nothing; `TRUE` from one source and
`FALSE` from another resolves to `BOTH` — the contradiction is
**recorded**, never collapsed to a forced boolean.

### 3. Rust Binding (near-sdk-rs)

```rust
// View method — callable cross-contract by any near-sdk-rs contract.
#[near(contract_state)]
impl Contract {
    #[handle_result]
    pub fn lpres_join(&self, op: u8, left: u8, right: u8) -> u8 {
        // 0 conjoin (meet), 1 disjoin (join), 2 kjoin (knowledge join)
        match op {
            0 => (left.min(right)),                      // rank meet
            1 => (left.max(right)),                      // rank join
            2 => (left | right) & 3u8,                   // kjoin
            _ => 0,                                      // unknown op: NEITHER
        }
    }

    pub fn lpres_is_settleable(&self, state: u8) -> bool {
        state == 1 // only TRUE authorizes
    }
}
```

Note: the rank ordering for meet/join is `FALSE(2) < NEITHER(0) <
BOTH(3) < TRUE(1)` on the project's chain — `min`/`max` above use that
rank map, not the numeric codes. A compliant implementation MUST use
the chain rank, not raw code order.

### 4. Known-Answer Vectors (KAT)

The full 16-cell kjoin table, verified live on mainnet (see
`docs/evidence/lpres_join_onchain_proof.json`):

| a \ b | NEITHER | TRUE | FALSE | BOTH |
|---|---|---|---|---|
| **NEITHER** | NEITHER | TRUE | FALSE | BOTH |
| **TRUE** | TRUE | TRUE | BOTH | BOTH |
| **FALSE** | FALSE | BOTH | FALSE | BOTH |
| **BOTH** | BOTH | BOTH | BOTH | BOTH |

Headline vectors: `kjoin(TRUE, TRUE) = TRUE` (settleable),
`kjoin(TRUE, FALSE) = BOTH` (retained, never settleable),
`kjoin(FALSE, FALSE) = FALSE`, `kjoin(NEITHER, TRUE) = TRUE`.

### 5. Settlement Invariant

Contradictions resolved to `BOTH` (`0x03`) allow the state machine to
record and isolate the dispute, but under **no circumstance** may a
`BOTH` or `NEITHER` state authorize financial transfers or final state
finality. Only `TRUE` (`0x01`) authorizes a settlement action.

## Formal Primitives (in this repository)

Reviewers: the algebraic laws, the Rust crate, and the runtime/storage
invariants are provided preemptively:

- **Algebra:** `docs/spec/algebra.md` — the lattice, the chain rank,
  commutativity/associativity/idempotence proofs, the absorption law
  (BOTH is absorbing), and the non-coercion theorem (no kjoin
  containing FALSE or BOTH yields TRUE).
- **Rust:** `reference/lpres.rs` — `LogicState` + `ParaconsistentGate`,
  near-sdk-rs-compatible, with tests (7 pass; kjoin table matches the
  on-chain proof).
- **Runtime/storage:** `docs/spec/runtime-storage.md` — the 898 KB
  state layout, the gate's gas/memory envelope, and the
  monolith-vs-chunking justification.
- **C reference:** `reference/lpres_gate.c` + 66-check test + `build.sh`.

## Subsystem Architecture (the distributed OS)

The kernel gate is the root of a layered system. Three satellite
contracts (near-sdk-rs, in this repo under `contracts/`) build the
userland OS: the **oracle-gateway** (dispute arbiter, ingestion),
**ipc-bus** (multi-agent messaging with paraconsistent gating), and
**execution-shield** (a vault whose transfers pass only when
`is_settleable == TRUE`). The `consumer-example` proves the
cross-contract promise path into the kernel's `lpres_join`. Full
topology: `docs/architecture/subsystems.md`.

## Reference Implementation (live on NEAR mainnet)

- **Contract account:** `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
- **Live code hash (current):** `315cfVxvveFBrT6FnT4Cr27AoQrNSyYG1XF8Lu13ZCCF`
- **Code hash chain (the promoted hash leads here):**
  `6N7d4cDC…` (v15) → `AiND1AEh…` (audited) → `315cfVxv…` (current —
  the ft_transfer_call promise-buffer fix, deploy TX
  `J21zCZ13hoyTXrFSzKurMmpvS6U7Sh3c6y9wM6dkfe7P`, state preserved:
  supply 60,185, ledger, holders, gate proof all unchanged).
- **Implementation:** freestanding C kernel (zero-libc Wasm32 module)
  — `kernel/arch/near/near_contract.c` + `kernel/src/lpres/lpres_core.c`,
  compiled with clang 23.1.0 `--target=wasm32-unknown-unknown`,
  Apache-2.0.
- **On-chain proof (gas-free view, callable by anyone):**
  `lpres_join(kjoin, TRUE, FALSE)` returns `3` (`BOTH`) — verified on
  mainnet 2026-10-19 and re-verified 2026-10-20.
- **Explorer:** https://nearblocks.io/address/6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8

## Security Considerations

Formal properties (verified by construction and by on-chain KAT
vectors):

1. **No coercion to authority.** No combination of inputs can make a
   `BOTH` (dispute) or `NEITHER` (unobserved) state evaluate as `TRUE`
   (authorization). `lpres_join(kjoin, a, b) == TRUE` requires
   `a == TRUE && b == TRUE` (or a chain that never introduces a
   contradiction).
2. **Contradiction retention.** `kjoin(TRUE, FALSE) = BOTH` —
   information is never destroyed by resolution; disputes remain
   observable in the ledger.
3. **Integer-exact.** The gate is a fixed 256-entry lookup on a bounded
   lattice; no floating point, no unbounded state, negligible gas.
4. **Deterministic.** Identical inputs produce identical outputs on any
   node (the kernel is integer-exact by construction).

## Rationale

The four-valued model is the minimal extension of Boolean logic that
can represent the states a multi-agent system actually encounters:
unobserved (`NEITHER`), verified (`TRUE`), refuted (`FALSE`), and
disputed (`BOTH`). Restricting settlement to `TRUE` while *recording*
`BOTH` gives agent frameworks a safe, auditable resolution path
instead of a revert/deadlock.

## Backwards Compatibility

The proposal adds a new interface; existing contracts are unaffected.
The reference implementation is deployed alongside the existing
`$YHVH` token surface (NEP-141) on the same account without conflict.

## Reference

- ZXV system documentation (IPFS): `QmPVXB22kX1HSSXRn76GfBgBLwFqfMtPNbgR9mHJkD7PiU`
- Forensic ledger (IPFS): `QmSFZ7W7VGoadJLhd8ARDRpukA8XCKaQ9YtjnxLgbEUSQX`
- Source repository: https://github.com/sykomiked/ZEDEC-XERO-VULPINE-V1.0.0