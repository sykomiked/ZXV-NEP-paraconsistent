---
NEP: 0000 # Assigned by NEP Editors upon PR submission
Title: Paraconsistent Settlement Layer as a Community Utility — Four-Valued Logic for the NEAR Ecosystem
Author: Michael Laurence Curzi <deal@zedec.ai> · ZXV (ZEDEC / 36N9 Genetics, LLC)
Status: Draft
Type: Standards Track (Contract Standards / Developer Tools)
Category: Contract
Created: 2026-10-23
Requires: NEP-141, NEP-171, NEP-297
Discussions-To: https://github.com/near/NEPs/discussions/XXXX
---

# Paraconsistent Settlement Layer as a Community Utility

## Summary

A four-valued paraconsistent settlement layer for NEAR, offered as an
**open community utility**: a logic kernel in which **only `TRUE`
settles**, contradictions (`TRUE + FALSE = BOTH`) are recorded, never
coerced, and never panic. The reference implementation is live on NEAR
mainnet as a public good — canonical address `zedec.near`, a 110-kit
micro-contract at
`f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`,
a 136-capability registry at
`7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`,
and the kernel gate at
`6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`.

This NEP specifies the minimum viable interface for any NEAR protocol,
DAO, oracle, AI agent, or human operator to interoperate with the
paraconsistent gate. It is written to be **deeply respectful of NEAR's
culture**: open, community-owned, gas-free to read, self-describing,
and buildable by anyone. It is not a commercial proposal — it is an
**enhancement to the protocol's expressive range**, making a new
logical primitive available to every project on NEAR as a public
utility and demonstration.

The companion public-utility repository (with the full user manual,
the multilingual documentation, and the demonstration guide) is
referenced in the References section and published at
`https://github.com/sykomiked/zxv-community-utility`.

---

## Table of Contents

1. Summary
2. Motivation — the binary assumption and the gap this NEP bridges
3. The paraconsistent foundation — what it is, why it matters
4. The invariant
5. The four-valued state space
6. The chain-meet algebra (the mathematics)
7. The minimum viable interface (MVI)
8. The reference implementation
9. The canonical router (zedec.near)
10. The universal interface (self-describing)
11. The user manual (embedded)
12. The economic perimeter
13. The governance layer
14. The Venice Gas Station
15. Security considerations
16. Backwards compatibility
17. The public-utility repository
18. The multilingual documentation (30 languages)
19. Frequently asked questions
20. References
21. Copyright

---

## 2. Motivation — the binary assumption and the gap this NEP bridges

### 2.1 The binary assumption

Most of the NEAR ecosystem — like most of the computing world —
operates on **binary logic**: every proposition is either true or
false; every contract call either succeeds or fails; every dispute
either resolves one way or the other. This assumption is so deep it is
rarely examined. Yet it is the root of four recurring failure modes in
decentralized systems:

| Failure mode | Classical behavior | What is lost |
|---|---|---|
| Two oracle feeds diverge | revert / deadlock / unfair liquidation | the disagreement itself |
| Two AI models disagree | panic or pick a side | the conflicting evidence |
| Two agents assert conflicting directives | undefined behavior | the audit trail of the conflict |
| Two communities propose opposing governance | fork or silence the minority | the coexistence of both views |

### 2.2 The gap this NEP bridges

**NEAR currently has no standard primitive for *holding* a
contradiction as a first-class state.** The ecosystem takes binary for
granted because binary is all it has been given. This NEP introduces
the missing option — a four-valued logic layer — so that projects
*can* choose to treat disagreement as information, not as a bug.

The four classical responses to disagreement all destroy information:

1. **Panic** — throw an exception, roll back state. The disagreement
   is discarded; the system restarts from before the conflict.
2. **Last-writer-wins** — the newer timestamp overwrites the older.
   The older truth is silently erased.
3. **Deadlock** — stop processing and wait. The system freezes, and
   the disagreement persists invisibly.
4. **Majority coercion** — the larger vote count forces a resolution.
   Power wins, not truth; the minority's evidence is discarded.

Paraconsistent logic offers a fifth path: **record the disagreement,
hold both claims, and refuse to settle until the evidence genuinely
resolves.** The contradiction becomes information — audit-trail
material — rather than a failure.

### 2.3 Why a community utility

NEAR's strength is its community: open-source, cooperative, and
builder-first. A paraconsistent gate is most valuable when it is a
**shared utility** — one well-audited, gas-free, self-describing
primitive that any project can call, rather than every project
re-deriving its own. This NEP proposes exactly that: a public
reference implementation, a minimal interface, and a developer kit,
all free and open.

The utility is presented in the spirit of NEAR's founding values:
**open access** (gas-free reads for everyone), **community ownership**
(anyone may call it, fork it, or propose changes through the standard
NEP process), and **public benefit** (the logic is a general-purpose
primitive, not a walled garden).

---

## 3. The paraconsistent foundation — what it is, why it matters

### 3.1 Classical logic and the principle of explosion

Classical two-valued logic assumes the law of excluded middle
($P \lor \neg P$): every proposition is either true or false. In a
distributed environment, network partitions and replication lag can
cause **evidence for both $P$ and $\neg P$ to arrive simultaneously**.
Within the classical framework, this triggers the **principle of
explosion**: from the contradiction $P \land \neg P$, any proposition
is derivable, and the entire inference system becomes meaningless.

This is why classical implementations "avoid" contradiction rather
than handling it: the logic itself gives them no safe place to put it.

### 3.2 Paraconsistent logic: the fifth path

Paraconsistent logic flips the premise:

> **A contradiction is not an error state. It is an information state.**

The system holds both claims, records the conflict, and refuses to
force a resolution until the evidence genuinely settles. This is not a
philosophical stance — it is a **mathematically well-defined
four-valued logic**, implementable as a state-machine primitive.

### 3.3 The four values, as a community of states

| State | Value | Meaning | Behavior |
|---|---|---|---|
| `NEITHER` | `0x00` | no evidence yet | waits |
| `TRUE` | `0x01` | agreed, attested | **settles** |
| `FALSE` | `0x02` | explicitly refuted | refuses |
| `BOTH` | `0x03` | contradiction, held | **recorded, never coerced** |

The four states are bit patterns in a single byte. `0x00`–`0x03` are
the core; the reference implementation extends to 145 states
(expansion phases by Fibonacci milestones) without changing the
operators — each new phase takes the next rank.

---

## 4. The invariant

The entire layer rests on one rule, expressible in every language:

> **Only TRUE settles. TRUE+FALSE=BOTH — contradictions are recorded,
> never coerced, never a panic.**

This is not a runtime check; it is a **theorem of the rank structure**.
Because TRUE sits at the top of the total order, no combination of
non-TRUE values can ever yield TRUE. The invariant is structural.

---

## 5. The four-valued state space

### 5.1 States as bit patterns

```c
/* lpres_phases.h — the canonical state definition */
typedef enum {
    LPRES_STATE_NEITHER = 0,  /* no evidence (unobserved, pending) */
    LPRES_STATE_TRUE    = 1,  /* agreed, true (settleable)         */
    LPRES_STATE_FALSE   = 2,  /* explicitly refuted (refused)      */
    LPRES_STATE_BOTH    = 3,  /* contradiction (both evidences)    */
    LPRES_STATE_GATEWAY = 4,  /* fifth value: the expansion port   */
    /* ... expansion phases by Fibonacci milestones ... */
    LPRES_STATE_COUNT   = 145 /* total states after expansion      */
} lpres_state_t;
```

### 5.2 The total order (chain rank)

The core design decision: **not Belnap's lattice, but a rank on a
totally ordered chain**.

```
FALSE (0) < NEITHER (1) < BOTH (2) < TRUE (3)
```

FALSE is the bottom; TRUE is the top.

### 5.3 The meet (conjoin) and join (disjoin)

```c
lpres_state_t lpres_conjoin(lpres_state_t a, lpres_state_t b) {
    /* the meet: min on the chain. FALSE dominates, TRUE is identity,
     * NEITHER absorbs BOTH — all by the rank, so a new phase needs only
     * its rank, never a new branch. Deliberate deviation from Belnap's
     * FOUR (the chain, not the lattice): no mix of non-TRUE values can
     * yield TRUE, the only value that authorises an action. */
    if ((unsigned) a >= LPRES_STATE_COUNT ||
        (unsigned) b >= LPRES_STATE_COUNT)
        return LPRES_STATE_NEITHER;   /* an unknown phase is never authority */
    return lpres_rank[a] <= lpres_rank[b] ? a : b;
}
```

| Input | Result | Meaning |
|---|---|---|
| `conjoin(FALSE, x)` | `FALSE` | FALSE is the dominator (bottom) |
| `conjoin(TRUE, TRUE)` | `TRUE` | only agreement settles |
| `conjoin(BOTH, TRUE)` | `BOTH` | contradiction is held, never coerced |
| `conjoin(NEITHER, BOTH)` | `NEITHER` | NEITHER absorbs BOTH |
| `conjoin(unknown, x)` | `NEITHER` | out-of-range guard (fail-closed) |

---

## 6. The chain-meet algebra (the mathematics)

### 6.1 The bounded lattice

The meet/join pair with the chain rank forms a **bounded lattice**.
Verified over all combinations (4×4=16, and 64 for associativity):

```
Meet: idempotent ○ commutative ○ associative ○
Join: idempotent ○ commutative ○ associative ○
Absorption: ○ (a ∧ (a ∨ b) = a)
Partial order: ○ (a ≤ b ⇔ meet(a,b) = a)
Invariant: ○ (a meet of non-TRUE values never yields TRUE)
```

### 6.2 The structural guarantee

Since TRUE has rank 3 (the top), the meet (which returns the minimum)
can be TRUE **only if both inputs are TRUE**. Therefore:

> **The invariant "only TRUE settles" is derived mathematically from
> the rank structure.** No runtime check enforces it, because no
> runtime state can violate it.

### 6.3 The deliberate deviation from Belnap

Belnap's `FOUR` is a lattice with two orders (information order and
truth order). The reference implementation collapses it into a single
totally ordered chain. Benefits:

- adding a phase requires only adding a rank (no new branch logic);
- the implementation reduces to a single integer comparison (`<=`);
- the "non-TRUE combination yields TRUE" problem possible in Belnap's
  lattice is structurally eliminated.

### 6.4 The conjoin table (full)

```
meet:       0    1    2    3   (columns = b)
  0:        0    0    2    0
  1:        0    1    2    3
  2:        2    2    2    2
  3:        0    3    2    3
```

(rows = a; values are the meet result; TRUE=1, FALSE=2, NEITHER=0,
BOTH=3; the table follows the rank order FALSE < NEITHER < BOTH <
TRUE.)

---

## 7. The minimum viable interface (MVI)

The reference implementation exposes the following interface. Any
NEAR contract can interoperate by calling these methods on
`zedec.near` (the canonical router) or the gate directly.

### The gate

```
ExtParaconsistentGate {
  /// Join two evidence states; the chain-meet. Returns 0x00-0x03.
  fn conjoin(inputs: Vec<u8>) -> u8;

  /// Only TRUE (0x01) is settleable.
  fn is_settleable(state: u8) -> bool;

  /// The invariant as a string, self-describing.
  fn invariant() -> String;
}
```

### The canonical router (zedec.near)

```
ExtCanonicalRouter {
  /// The full protocol topology: registry, gate, core.
  fn route() -> String;

  /// The public protocol record.
  fn protocol_meta() -> String;

  /// The invariant check.
  fn is_settleable(state: u8) -> bool;
}
```

### The registry (discovery)

```
ExtRegistry {
  /// All registered capabilities (currently 136).
  fn vm_capabilities() -> Vec<String>;

  /// Resolve a capability name to its contract.
  fn vm_capability(name: String) -> Option<AccountId>;
}
```

---

## 8. The reference implementation

### 8.1 Addresses

- **Canonical (routing):** `zedec.near` — a live contract answering
  `route()`, `protocol_meta()`, and `is_settleable()`.
- **The protocol (110 kits):**
  `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`
- **The registry (136 capabilities):**
  `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`
- **The gate (kernel invariant):**
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`

### 8.2 The gate code (excerpt)

```rust
#[near_bindgen]
impl ZxvCore {
    pub fn conjoin(&self, inputs: &[u8]) -> u8 {
        let mut state = 0u8; // NEITHER
        for v in inputs {
            state = meet(state, *v);
        }
        state
    }
    pub fn is_settleable(&self, state: String) -> bool {
        state == "TRUE"
    }
}
```

### 8.3 Gas efficiency

- `conjoin` / `is_settleable`: under 1.2 Tgas (internal envelope bound)
- whole execution: under 10 Tgas
- **reads (view calls): entirely gas-free**

### 8.4 Quality record (measured, not claimed)

| Check | Result |
|---|---|
| QC harness (vs the kernel C source) | 55/55, zero bugs |
| On-chain audit (all layers) | 24/24 |
| Durability stress | 70/70 valid calls, zero failures |
| Registration audit | 136/136 resolve to live contracts |
| Gas-wall test (reads / writes) | 125/125 / 10/10 |

---

## 9. The canonical router (zedec.near)

The canonical address `zedec.near` is a **live contract** that routes
to the entire protocol. It answers:

```
zedec.near → route() → canonical=zedec.near|registry=7eaa2c6f...|gate=6ca890f7...|core=f18889ea...
zedec.near → protocol_meta() → primaryWallet=zedec.near|protocolAddress=f18889ea...|...
zedec.near → is_settleable(1) → true
```

`zedec.near` is the community's front door: one address that points to
the whole utility, readable by any client, gas-free.

---

## 10. The universal interface (self-describing)

Any client — the simplest bot, the most advanced AI, the best human —
reads the same interface, gas-free:

```bash
# the manifest: the whole system's schema
#   ux_manifest on f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf
# the help: any topic
#   ux_help {"topic":"start"}  → the 5-step onboarding
#   ux_help {"topic":"onboard"} → your first grant
#   ux_help {"topic":"gas"}    → the Venice Gas Station
#   ux_help {"topic":"sd"}     → the Sovereign Decrees
# the registry (discovery)
#   vm_capabilities on 7eaa2c6f... → 136 capabilities
# the invariant (verification)
#   conjoin {"inputs":[1,2]}        → "FALSE"
#   is_settleable {"state":"BOTH"}  → false
#   is_settleable {"state":"TRUE"}  → true
```

---

## 11. The user manual (embedded)

This NEP doubles as the user manual for the public utility. The
complete manual — how to read, join, verify, and build on the system —
is embedded here and mirrored in the companion repository.

### 11.1 What this is

A public, gas-free logic utility on NEAR: a gate that answers
"is this state settleable?" with the paraconsistent invariant, plus a
registry that makes every capability discoverable, plus a canonical
router (`zedec.near`) that points to all of it.

### 11.2 How to read it (any client, no wallet)

```bash
curl -s https://rpc.mainnet.fastnear.com -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"zedec.near","method_name":"route","args_base64":"e30="}}'
```

### 11.3 How to verify the invariant

```bash
# is_settleable {"state":"BOTH"}  → false (a contradiction never settles)
# is_settleable {"state":"TRUE"}  → true  (agreement settles)
# verify_chain {"from":0}         → true  (the audit hash-chain is intact)
```

### 11.4 How to join

A new actor calls `mint_onboard` for a one-time grant (spendable on
the platform, not transferable — one per account). Then `pool_enter`
provides access through the liquidity pool.

### 11.5 How to build on it

- **The contract IS the API** — serverless, gas-free to read.
- **The bridge is a standard** — any relayer implements it.
- **The developer kit** — Rust crate `zxv-sdk` + TypeScript
  `@zxv/developer-kit`.
- **The SDK repositories** — see References.

### 11.6 The full capability surface (136 capabilities)

The registry lists 136 capabilities, including:

- **28 base kits**: gate, treasury, rails, receipts, vino, tribute,
  surplus, chronicle, allotment, broker, rmag, scp144, aipi, chiglet,
  count_house, vswap, vauction, vbond, identity, reputation,
  provenance, grants, community_chest, royalty, telemetry, market,
  intent, mesh_token
- **5 layers**: settlement, market, trust, distribution, agent
- **7 systems**: escrow, agent_economy, governance, marketplace,
  compliance, identity_verification, settlement_rail
- **4 meta**: orchestrator, router, self, adaptation
- **4 api**: gateway, verify, bridge, index
- **4 ux**: manifest, help, example, universal
- **11 rewards**: tiers, accrual, claim, referral, streak,
  achievements, leaderboard, distribution, agent, round, bonus
- **10 player**: cards, quests, badges, levels, xp, missions,
  trophies, leaderboard, loyalty, events
- **10 ops**: config, limits, timelock, pause, audit, health, metrics,
  upgrade, guard, schedule
- **the rest**: the gas station, the system icon NFT, the Port of
  Neo-Sicily, the YHVH circulation, the liquidity pool, the Sovereign
  Decrees, the canonical router, and the discovery names

### 11.7 The 17 services (api_services)

gate.v1 · treasury.v1 · rails.v1 · receipts.v1 · oracle.v1 · ai.v1 ·
market.v1 · identity.v1 · meta.v1 · bridge.v1 · gas.v1 · rewards.v1 ·
player.v1 · ops.v1 · port.v1 · pool.v1 · onboard.v1

### 11.8 The onboarding flow (detailed)

1. Call `ux_help {"topic":"start"}` — the system teaches the 5-step
   onboarding.
2. Call `mint_onboard {}` — receive the one-time 5000-unit grant
   (spendable-not-transferable, one per account).
3. Call `ux_help {"topic":"port"}` — learn to register a DAO in the
   Port of Neo-Sicily.
4. Call `ux_help {"topic":"pool"}` — learn to enter through the
   liquidity pool.
5. Call `ux_help {"topic":"sd"}` — learn the Sovereign Decrees.

### 11.9 The verification flow (detailed)

```bash
# the invariant
conjoin {"inputs":[1,2]}        → "FALSE"
is_settleable {"state":"BOTH"}  → false
is_settleable {"state":"TRUE"}  → true
# the audit
verify_chain {"from":0}         → true
# the discovery
vm_capabilities on the registry → 136
```

---

## 12. The economic perimeter

The utility is designed around **work-based participation**,
consistent with regulatory best practices:

- rewards accrue per **verified action**, never to idle capital;
- a TRUE settlement receives a 2x protocol gas match (the Venice Gas
  Station); a BOTH forfeits the match (the drain shield) — so
  contradiction cannot be spammed;
- the onboarding grant is spendable-not-transferable, preventing
  wallet-farming;
- the epoch issuance is bounded at 2% of circulating supply per year.

### 12.1 The treasury split

The work-based economy distributes by the 50/25/15/10 rule: 50% to
the worker, 25% to the buffer, 15% to infrastructure, 10% to
regeneration. Workers post slashable performance bonds; idle capital
earns nothing.

### 12.2 The YHVH circulation

- onboarding grant: 5000, spendable-not-transferable, one per account
- epoch issuance: 2% of circulating per year, rate-bounded
- work bounties: per verified action (aipi_quorum 100, port_dao 200)

---

## 13. The governance layer (Sovereign Decrees)

The utility includes a governance primitive that embodies the
paraconsistent philosophy: **forks without forking**. A person issues a
Sovereign Decree; it applies to them and those who sign on.
Contradictory decrees coexist; adherents shift freely. The chain never
forks because the logic holds both views as information.

- `sd_issue` — issue a decree (applies to you + signers)
- `sd_sign` — sign a decree (the registry of adherents)
- `sd_shift` — switch your adherence (back and forth, freely)
- `sd_prefabs` / `sd_choose` — ready configurations (6 prefabs)
- `sd_require` / `sd_check` — protocols gate participation on decree
  families
- `sd_forkless` — proof that contradictory decrees coexist

---

## 14. The Venice Gas Station

The utility includes a self-funded execution envelope: users deposit
into a dedicated gas envelope; a TRUE settlement receives a 2x
protocol gas match (half from the user, half from the protocol's 15%
infrastructure reserve); a BOTH or FALSE forfeits the match (the drain
shield). Deposits are never rehypothecated.

| Settlement state | Gas burden | Protocol match |
|---|---|---|
| `TRUE` (settled) | user 50% | protocol 50% (effective 2x) |
| `BOTH` (contradiction) | user 100% | none (drain shield) |
| `FALSE` (refused) | user 100% | none |

---

## 15. Security considerations

- The naive knowledge-join (`(a|b) & 3`) returns TRUE for out-of-range
  input; the reference implementation uses the range-checked
  `lpres_conjoin` instead. Only `lpres_conjoin` is used for settlement
  evidence.
- The out-of-range guard is fail-closed: an unknown phase is never
  authority.
- The struct is never mutated across upgrades; state-compatible
  upgrades (Borsh forward compatibility) mean deployed state is never
  broken.
- The contract is open source; the QC harness cross-checks the logic
  against the kernel C source.
- The economic perimeter is work-based, not speculative: no passive
  yield, no rehypothecation, bounded issuance.

---

## 16. Backwards compatibility

No breaking changes to NEAR Core, NEP-141, NEP-171, or NEP-297. The
layer is an opt-in settlement guard: existing contracts continue to
work exactly as before; new contracts may call the gate when they
choose.

---

## 17. The public-utility repository

The companion repository,
`https://github.com/sykomiked/zxv-community-utility`, contains:

- the user manual (the complete guide to the ecosystem)
- the demonstration guide (how to verify the invariant yourself)
- the multilingual documentation (30 world languages, omni-tautology
  keyword structure)
- the NEP reference (this document)
- the architecture overview

It is presented as a **public utility and demonstration**: anyone can
read, verify, and build on it, without any commercial obligation.

---

## 18. The multilingual documentation (30 languages)

The companion repository includes the same documentation in the top
30 world languages — English, 简体中文, हिन्दी, Español, Français,
العربية, বাংলা, Português, Русский, اردو, Bahasa Indonesia, Deutsch,
日本語, Kiswahili, मराठी, తెలుగు, Türkçe, தமிழ், Tiếng Việt, 한국어,
Italiano, ไทย, ગુજરાતી, Basa Jawa, Polski, Українська, Bahasa Melayu,
فارسی, ಕನ್ನಡ, ਪੰਜਾਬੀ — each written natively, each unique, forming an
omni-tautology of keywords so the utility is findable in every
language. The invariant appears in each: "Only TRUE settles."

---

## 19. Frequently asked questions

### Q: Is this a token or a commercial product?
A: No. This NEP proposes a **logic primitive** as a community utility.
The reference implementation is open source, gas-free to read, and
offered for public benefit. It is a demonstration of what four-valued
logic enables on NEAR.

### Q: Do I have to use it?
A: No. It is opt-in. Existing contracts are unaffected.

### Q: What does it cost to read?
A: Nothing. All reads are gas-free view calls.

### Q: How do I verify the claims in this NEP?
A: Directly on-chain, gas-free: call `is_settleable`, `conjoin`,
`verify_chain`, and `vm_capabilities` on the addresses above.

### Q: What is `zedec.near`?
A: The canonical router: a live contract that points to the whole
utility (the protocol, the registry, and the gate).

### Q: How is this different from Belnap's logic?
A: The reference implementation uses a single totally ordered chain
(FALSE < NEITHER < BOTH < TRUE) rather than Belnap's two-order
lattice, so the invariant is structural and the implementation is a
single comparison.

---

## 20. References

- **The public-utility repository (this NEP's companion):**
  https://github.com/sykomiked/zxv-community-utility
- **The stack:** https://github.com/sykomiked/zxv-stack
- **The developer kit:** https://github.com/sykomiked/zxv-developer-kit
- **The NEP + ledger:** https://github.com/sykomiked/ZXV-NEP-paraconsistent

---

## 21. Copyright

Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC ·
SPDX-License-Identifier: Apache-2.0