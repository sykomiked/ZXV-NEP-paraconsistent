---
NEP: 0000 # Assigned by NEP Editors upon PR submission
Title: Standard Interface for Deterministic Panic-Free State Resolution (Chain-Meet)
Author: Michael Laurence Curzi <deal@zedec.ai> · ZXV (ZEDEC / 36N9 Genetics, LLC)
Status: Draft
Type: Standards Track (Contract Interface)
Category: Contract
Created: 2026-10-23
Requires: NEP-141, NEP-171, NEP-297
Discussions-To: https://github.com/near/NEPs/discussions/XXXX
---

# Standard Interface for Deterministic Panic-Free State Resolution (Chain-Meet)

## Summary

A standard contract interface for **non-aborting state resolution**
when asynchronous execution produces conflicting evidence. The standard
defines a 1-byte contention state (`BOTH`, `0x03`) that **isolates
conflict, guarantees audit preservation, and structurally blocks
side-effects** — instead of invoking `env::panic_str()`.

The interface is a **zero-cost invariant**:
- **1 byte** per state (`0x00`–`0x03`)
- **O(1)** resolution (a scalar integer comparison)
- **0 heap allocations** (`#![no_std]` compatible)
- **< 1.2 Tgas** on-chain; **gas-free** as RPC view calls

A **working mainnet implementation** is live at `zedec.near` and
`zxv-core` (`f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`),
having survived adversarial mainnet load. This NEP standardizes that
existing interface for the rest of the ecosystem.

---

## 1. The Problem: `env::panic_str()` is a Fragile Fallback

### 1.1 The operational failure mode

When cross-shard promises or multi-oracle attestations conflict within
an asynchronous execution cycle, the standard response is
`env::panic_str()`. The consequences:

1. **Intermediate receipts roll back** — work already completed in the
   execution cycle is discarded.
2. **Prepaid gas burns without actionable diagnostics** — the panic
   message is often the only artifact, and it is not structured.
3. **Downstream automation pipelines halt** — a panicking contract
   stops the entire intent/automation chain, not just the disputed
   step.

### 1.2 Why this is a standards problem, not a project problem

Every NEAR contract that consumes multiple oracles, aggregates agent
intents, or awaits cross-shard callbacks faces this. There is currently
**no standard primitive** for recording "these two sources disagree,
hold the funds, preserve the evidence, do not revert." Each project
either panics (losing state) or implements a bespoke last-writer-wins
(LWW) (losing evidence). Both are losses.

---

## 2. The Proposal: A 1-Byte Non-Aborting Contention State

### 2.1 The interface

```rust
// The standard interface — panic-free state resolution
pub trait PanicFreeResolution {
    /// Conjoin two evidence states; the chain-meet. Returns 0x00-0x03.
    /// O(1): a single scalar integer comparison.
    fn nep_conjoin(inputs: &[u8]) -> u8;

    /// Only TRUE (0x01) is settleable. BOTH (0x03) blocks side-effects
    /// without reverting.
    fn nep_is_settleable(state: u8) -> bool;

    /// The invariant, self-describing.
    fn nep_invariant() -> String;
}
```

### 2.2 The state encoding

| State | Value | Meaning | Side-effects |
|---|---|---|---|
| `NEITHER` | `0x00` | no evidence yet | blocked |
| `TRUE` | `0x01` | agreed, attested | **allowed** |
| `FALSE` | `0x02` | explicitly refuted | blocked |
| `BOTH` | `0x03` | contradiction, held | **blocked, but NOT reverted** |

The four states are bit patterns in a single byte. No serialization,
no nested allocation, no runtime math.

### 2.3 The resolution rule (chain-meet)

```rust
const RANK: [u8; 4] = [1, 3, 0, 2]; // [NEITHER, TRUE, FALSE, BOTH]

fn meet(a: u8, b: u8) -> u8 {
    if a > 3 || b > 3 { return 0; }   // fail-closed: unknown is never authority
    if RANK[a as usize] <= RANK[b as usize] { a } else { b }
}
```

The total order is `FALSE < NEITHER < BOTH < TRUE`. Because TRUE is the
top of the chain, **no combination of non-TRUE values can ever yield
TRUE** — the invariant is structural, not checked.

---

## 3. Why This Passes the Four Working-Group Criteria

### 3.1 Gas Predictability

- Resolution is **O(1)**: a single `min_rank` scalar comparison.
- On-chain: `nep_conjoin` and `nep_is_settleable` execute **well under
  1.2 Tgas**.
- Off-chain: **entirely gas-free** as RPC view calls.
- No loops over nested state, no serialization, no heap.

### 3.2 Cross-Contract Reliability

- A `BOTH` state **blocks side-effects without reverting** the
  contract.
- The receipt hash is **retained** for the next block — the audit
  trail is preserved, not discarded.
- Downstream contracts can read the contention state and schedule
  arbitration, instead of catching a panic.

### 3.3 Asynchronous Composability

- Cross-shard callbacks that return conflicting execution receipts
  resolve to `BOTH` instead of panicking.
- The parent contract **keeps its completed steps** and records the
  conflict.
- The state is **1 byte**, so it composes into any receipt, event, or
  log without overhead.

### 3.4 Storage Overhead

- **1 byte** per state value.
- **0 heap allocations** — `#![no_std]` compatible.
- No new top-level struct fields required; the state lives as a `u8`.
- Storage cost is effectively zero.

---

## 4. The NEAR 2026 Narrative: AI Agents & Intent Settlement

NEAR's strategic direction centers on **AI Agents, Intent Settlement,
and Chain Abstraction**. Autonomous agents routing intents across
chains and shards constantly encounter contradictory off-chain inputs
and divergent oracle feeds.

### 4.1 The agent failure mode

When autonomous agents or solver networks submit mismatched state
proofs for an intent, a classical contract fails or gets caught in a
race condition:

- Agent A asserts `TRUE` for a settlement condition.
- Agent B asserts `FALSE` for the same condition.
- Classical response: panic, or last-writer-wins.

### 4.2 The deterministic intent conjoinment

This interface provides **deterministic intent conjoinment**: multiple
AI solvers or oracles can assert states simultaneously.

- **Solvers agreeing** → evaluates to `TRUE` → **settlement unlocked**.
- **Solvers disputing** → evaluates to `BOTH` → **funds remain locked
  in contract escrow without transaction reversal**, enabling
  automated arbitration without gas-draining panic loops.

This is the missing primitive for intent settlement: a way to hold a
dispute **inside** the escrow, rather than exploding the transaction.

---

## 5. Concrete Use Cases (Rationale)

### 5.1 Multi-Oracle Price Feeds

**Scenario:** Oracle A reports an asset status; Oracle B disputes it.

**Without this interface:** the contract panics (reverting the call)
or silently overwrites via last-writer-wins (LWW) — losing Oracle A's
evidence.

**With this interface:** the contract automatically records `BOTH`.
No panic, no overwrite. The dispute is held, the audit trail is
preserved, and an arbitration path can resolve it.

### 5.2 Multi-Sig Escrow / Sovereign Decrees

**Scenario:** signers submit opposing directives for an escrow release.

**Without this interface:** conflicting votes break transaction
continuity.

**With this interface:** conflicting votes are acknowledged and
preserved in the append-only state trie. The escrow stays intact; the
conflict is data, not a failure.

### 5.3 Cross-Shard Call Recovery

**Scenario:** an asynchronous callback from another shard returns
conflicting execution receipts.

**Without this interface:** the parent contract reverts prior completed
steps.

**With this interface:** the parent contract resolves to `BOTH`,
retaining the receipt hash for the next block without reverting
completed work. The next block can arbitrate.

---

## 6. The Working Proof-of-Concept (De Facto Standard)

Most NEPs are rejected or delayed because they are purely theoretical.
This standard has a **live mainnet implementation** that has already
survived adversarial load.

### 6.1 The live addresses

- **Canonical (routing):** `zedec.near`
- **The protocol (110 kits):**
  `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`
- **The registry (136 capabilities):**
  `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`
- **The gate (kernel invariant):**
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`

### 6.2 Verify the zero-allocation execution (gas-free view calls)

```bash
# the invariant — a contradiction never settles
curl -s https://rpc.mainnet.fastnear.com -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf","method_name":"is_settleable","args_base64":"eyJzdGF0ZSI6IkJPVEgifQ=="}}'
# → false

# agreement settles
#   is_settleable {"state":"TRUE"}  → true

# the canonical router
#   zedec.near → route() → the full topology
```

### 6.3 The verification record

| Check | Result |
|---|---|
| Input fuzzing (65,536 runs, whole u8 domain) | 0 violations |
| Gas-wall stress (continuous adversarial load) | 125/125 reads, 10/10 writes |
| QC harness (vs the kernel C source) | 55/55, zero bugs |
| On-chain audit (all layers) | 24/24 |
| Registration audit | 136/136 resolve to live contracts |

The system has survived **mainnet adversarial load**. This shifts the
review question from "Is this technically viable?" to "Should we
standardize this existing interface for the rest of the ecosystem?"

---

## 7. Specification

### 7.1 Method signatures

```rust
pub trait ExtPanicFreeResolution {
    /// Conjoin evidence states; the chain-meet (O(1)). Returns 0x00-0x03.
    /// Out-of-range inputs are fail-closed to NEITHER (0x00).
    fn nep_conjoin(&self, inputs: &[u8]) -> u8;

    /// Only TRUE (0x01) is settleable.
    fn nep_is_settleable(&self, state: u8) -> bool;

    /// The invariant, self-describing.
    fn nep_invariant(&self) -> String;
}
```

### 7.2 View-call semantics

- `nep_conjoin` and `nep_is_settleable` are **view calls**: gas-free
  when invoked via RPC.
- On-chain execution: `< 1.2 Tgas` for the internal envelope bound.
- The state is a `u8`; no serialization required.

### 7.3 Event emission

When a contract transitions to `BOTH`, it SHOULD emit an NEP-297
`EVENT_JSON` event so indexers and downstream automation can observe
the contention:

```
EVENT_JSON: {"standard":"nep-panic-free-resolution","version":"1.0.0",
"event":"contention_held","data":{"state":"BOTH","receipt_hash":"0x..."}}
```

---

## 8. Security Considerations

- The naive knowledge-join (`(a|b) & 3`) returns TRUE for out-of-range
  input; the standard uses the **range-checked chain-meet**
  (`min_rank`) instead. Only `min_rank` is used for settlement
  evidence.
- The out-of-range guard is **fail-closed**: an unknown phase is never
  authority.
- `BOTH` blocks side-effects **without reverting** — this is the
  intended semantics; implementers must ensure their state machine
  handles a held state (funds locked, evidence retained) rather than
  expecting a revert.
- The reference implementation is open source; the QC harness
  cross-checks the logic against the kernel C source.

---

## 9. Backwards Compatibility

No breaking changes to NEAR Core, NEP-141, NEP-171, or NEP-297. The
interface is **opt-in**: existing contracts continue to work exactly as
before; new contracts may implement `nep_conjoin` /
`nep_is_settleable` when they choose.

---

## 10. Reference Implementation Repositories

- **The public-utility repository (this NEP's companion):**
  https://github.com/sykomiked/zxv-community-utility
- **The stack:** https://github.com/sykomiked/zxv-stack
- **The developer kit:** https://github.com/sykomiked/zxv-developer-kit
- **The NEP + ledger:** https://github.com/sykomiked/ZXV-NEP-paraconsistent

---

## 11. Copyright

Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC ·
SPDX-License-Identifier: Apache-2.0