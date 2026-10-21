# ZXV — The Paraconsistent World Computer

## A White Paper

**Version 1.0 — 2026-10-21**
**Author: Michael Laurence Curzi (ZEDEC / 36N9 Genetics, LLC)**

---

> *"I made every feature and programmed every module for a reason. They're
> all for a purpose in the paraconsistent logic of this architecture.
> Build it like a paraconsistent system. But we need to find all the ones
> that aren't."*

---

## Abstract

ZXV is a vertically integrated, sovereign computing system built on a
single idea: **contradictions are information, not failures.** Classical
two-valued logic treats a contradiction as an explosion — a contract
reverts, a bridge halts, an oracle deadlocks. ZXV replaces that with a
four-valued logic in which a contradiction is *recorded, isolated, and
held* until evidence resolves it — and **only unambiguous truth
authorizes value movement.**

This white paper describes the complete ZXV stack: a freestanding
zero-libc C kernel running on the NEAR blockchain, a family of satellite
contracts providing settlement, arbitration, and price harmonization,
a nine-server content mesh, and content-addressed storage — together
forming a **paraconsistent world computer** whose settlement is safe by
construction.

---

## 1. The Problem

Modern decentralized systems fail at the boundary of disagreement:

| Failure mode | What happens |
|---|---|
| **Oracle divergence** | two price feeds disagree → contracts revert, liquidations fire unfairly |
| **Bridge disagreement** | two validators report different states → the bridge halts |
| **Agent conflict** | two AI agents assert contradictory directives → undefined behavior |
| **Insurance dispute** | two claim validators disagree → payout deadlocks |
| **Settlement ambiguity** | a payment's validity is contested → the whole transaction blocks |

Every one of these is a **logical contradiction** — and every classical
system's answer is the same: *panic, revert, or deadlock*. The principle
of explosion means one contradictory piece of evidence can destroy an
entire transaction.

ZXV's answer: **do not resolve contradictions by force. Record them.**

---

## 2. The Paraconsistent Foundation

### 2.1 The Four States

ZXV's kernel implements a four-valued logic (a Belnap–Dunn style
bilattice with ZXV's chain ordering):

| State | Value | Meaning |
|---|---|---|
| `NEITHER` | `0x00` | no evidence yet |
| `TRUE` | `0x01` | attested, agreed |
| `FALSE` | `0x02` | explicitly refuted |
| `BOTH` | `0x03` | contradiction retained |

### 2.2 The Knowledge Join (`kjoin`)

The kernel's core operation:

```
kjoin(a, b) = (a | b) & 0x03
```

| `kjoin` | NEITHER | TRUE | FALSE | BOTH |
|---|---|---|---|---|
| **NEITHER** | NEITHER | TRUE | FALSE | BOTH |
| **TRUE** | TRUE | TRUE | BOTH | BOTH |
| **FALSE** | FALSE | BOTH | FALSE | BOTH |
| **BOTH** | BOTH | BOTH | BOTH | BOTH |

The two properties that matter:

1. **Contradiction retention:** `kjoin(TRUE, FALSE) = BOTH`. A
   contradiction is absorbed and isolated — never coerced to one side,
   never a deadlock.
2. **BOTH is absorbing:** once a state is `BOTH`, more evidence of the
   same kind cannot coerce it. A majority cannot override a recorded
   contradiction. Disputes resolve only through evidence, never votes.

### 2.3 The Settlement Invariant

The security-critical rule, enforced by the kernel and every satellite:

> **Only `TRUE` is settleable. `BOTH` and `NEITHER` must never authorize
> financial settlement.**

`FALSE` explicitly refuses. `BOTH` holds. `NEITHER` waits. Only
unambiguous `TRUE` moves value. This single invariant is what makes
paraconsistent settlement *safe* rather than merely *tolerant*.

### 2.4 The Operator Codes

The kernel exposes three operators:

| Op | Name | Semantics |
|---|---|---|
| `0` | `conjoin` | meet (chain rank: FALSE < NEITHER < BOTH < TRUE) |
| `1` | `disjoin` | join (existential, never settlement) |
| `2` | `kjoin` | knowledge join `(a|b)&3` |

---

## 3. The Architecture

### 3.1 The Kernel — a Freestanding C Contract on NEAR

The heart of ZXV is a **zero-libc freestanding C kernel** compiled to
WebAssembly and deployed on NEAR mainnet:

- **No standard library, no near-sdk-rs** — a hand-written kernel with
  raw host bindings (`args_bytes`, `storage_*`, `env_*`), a deliberate
  no-parsing design for its binary ABI
- **Monolithic state** — ledger, holders, and the four-valued gate live
  in one deterministic state machine
- **Byte-verifiable** — the deployed wasm is reproducible from the C
  source; the promoted code hash is pinned and on-chain

The kernel's core surfaces:

| Surface | Role |
|---|---|
| `lpres_join` | the four-valued ALU (`op, left, right` raw bytes → state) |
| `ft_*` | the token ledger ($YHVH, NEP-141-compatible) |
| `ledger_state` / `economy` | the settlement state |
| `voucher_*` | the VINO floating-voucher economy |

**Deployment (mainnet):** account `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`, code hash `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB` — the audited, promoted artifact.

### 3.2 The Satellite Fabric

Around the kernel, a family of idiomatic near-sdk-rs contracts (9
registered and discoverable on mainnet):

| Contract | Role |
|---|---|
| **World Registry** | the boot sector — `vm_info()`, `vm_capabilities()`, subsystem discovery |
| **Oracle Gateway** | records conflicting claims as `BOTH` without reverting |
| **IPC Bus** | isolates incompatible task assertions |
| **Execution Shield** | funds move only on `TRUE` |
| **Micro-Infer** | deterministic integer-only on-chain AI (64/64 KAT) |
| **Model Registry** | content-addressed model manifests (no weights on-chain) |
| **Consumer** | the cross-contract integration proof |

Discovery is the boot contract's job: any protocol probing `vm_info()`
enumerates every registered capability and resolves real accounts —
**the system is self-describing; it does not need to be advertised.**

### 3.3 The New DeFi Layer (testnet-proven, mainnet-ready)

Three contracts extend the kernel into public DeFi infrastructure:

#### 3.3.1 The Paraconsistent Oracle (the price harmonizer)

Connects to **existing oracle feeds** and harmonizes them with the
kernel's gate:

```
feeds agree (within tolerance)  → TRUE   → a settleable price
feeds diverge                   → BOTH   → price flagged, settlement held
no feed                         → NEITHER → not usable
all refute                      → FALSE  → refused
```

Any protocol already using a price oracle calls `price(asset)` here
instead and gets a **dispute-tolerant answer** — a drop-in replacement
with added safety. Divergent prices never trigger liquidations or
payouts. Adopted instantly because it harmonizes what already exists.

#### 3.3.2 The Settlement Router (the receipt rail)

The standard dispute-and-settlement service any contract, DAO, or agent
calls with contradictory evidence:

```
submit_evidence(claim, source, value)  → kjoin accumulates
verdict(claim)                         → NEITHER/TRUE/FALSE/BOTH
settle(claim)                          → ONLY on TRUE; emits a receipt
```

Settlement receipts (`zxv-receipt-NNNNNN`) are the world-computer proof
— content-addressed, on-chain, verifiable. **BOTH is absorbing: a
majority cannot coerce a recorded contradiction** (proven on testnet).

#### 3.3.3 The YHVH Wrapper (the liquidity adapter)

A standard near-sdk NEP-141 token, `wYHVH`, backed 1:1 by real $YHVH:

```
kernel.ft_transfer(YHVH → wrapper)   ← the kernel's working path
wrapper mints wYHVH 1:1              ← standard, correct ABI
wYHVH → DEX pool                     ← seedable, tradeable
wYHVH → wrapper → YHVH               ← redeemable 1:1
```

This is the **scenic bypass**: the promoted kernel's `ft_transfer_call`
has a host-ABI defect (it passes `gas` into the `amount_ptr` slot, so
the host reads out of bounds). Rather than modify the promoted contract
— whose hash is pinned and public — the wrapper routes around the
defect using the kernel's *working* `ft_transfer`, giving $YHVH real
liquidity without changing the promoted hash.

### 3.4 The Armada — a Nine-Server Content Mesh

Nine regional servers (IND, EU, JPN, US-c, AUS, UK, US-e, US-w, SIN)
run the ZXV kernel as their serving OS:

- Each node serves `/zxv_boot.wasm` (the kernel engine)
- Each node carries a CISEX content index (18,958+ entries) — the boot
  manifest, forensic ledger, and documentation are anchored on all nine
- The mesh is a **content-addressed backbone**: regional redundancy for
  the protocol's canonical artifacts

### 3.5 Content-Addressed Storage

All canonical artifacts are pinned on IPFS via Filebase:

- Boot manifest, forensic ledger, code-migration notes, verifier
  scripts, the white paper — every artifact resolves on the public
  gateway (verified 200)
- The RSS telemetry feeds (see §6) carry the CID layer, so the content
  is discoverable *and* verifiable

---

## 4. The Economy

### 4.1 $YHVH

The kernel mints and tracks $YHVH (supply 60,185) — a NEP-141-compatible
token whose settlement is gated by the paraconsistent invariant.

### 4.2 The VINO Economy

- **V-Bills** — the economy's debt instruments, redeemable at par
- **Vouchers** — floating-value settlement instruments
- **Tributes / Fountain** — the distribution rails

### 4.3 The Utility Flywheel

The coin's value comes from what it is *needed for*:

```
services charge YHVH (verdicts, harmonized prices)
  → people hold YHVH to use the network
  → providers earn YHVH for work
  → earnings flow into the treasury
  → treasury backs V-Bills (real collateral)
  → V-Bills redeemable at par = confidence
  → more users → more fees → more backing
```

---

## 5. Security Model

| Invariant | Enforcement |
|---|---|
| Only `TRUE` settles | the kernel gate + every satellite |
| `BOTH` never coerced | the absorbing kjoin |
| No integer/float money | exact integer arithmetic only |
| No unbounded state | bounded rings, checked capacity |
| Byte-reproducible kernel | pinned source → wasm → hash |
| Rate-limit discipline | all RPC traffic throttled (429 backoff) |
| Discovery is authoritative | `vm_capability()` reads registered state, never hardcoded placeholders |

---

## 6. Verifiability & Telemetry

Every claim in this paper is publicly verifiable:

- **On-chain:** zero-gas view calls to the kernel (`lpres_join`,
  `ft_total_supply`, `ledger_state`) and satellites (`vm_info`,
  `infer`, `check`)
- **Known-answer vectors:** micro-inference deterministic 64/64 KAT
- **The RSS feeds:** four syndicated feeds (WebSub + Ping-O-Matic +
  Internet Archive snapshots) carrying NEAR validation logs, OS
  telemetry, and CID resolution status
- **The verifier:** `tools/verify_code_hash.py` proves the kernel's
  code-hash chain from the chain itself

---

## 7. Roadmap

| Phase | Status |
|---|---|
| Kernel on NEAR mainnet (gate, ledger, token) | ✅ live |
| 9-contract satellite fabric, 9/9 registered | ✅ live |
| Paraconsistent Oracle (harmonizer) | ✅ testnet-proven |
| Settlement Router (receipt rail) | ✅ testnet-proven |
| YHVH Wrapper (pool scenic bypass) | ✅ testnet-proven |
| Armada nine-node mesh + RSS telemetry | ✅ live |
| DeFi layer on mainnet | ⏳ awaiting funding |
| Pool liquidity (wYHVH/wNEAR) | ⏳ awaiting funding |
| The community flywheel | ⏳ distribution |

---

## 8. Conclusion

ZXV is not a logic toy. It is the **settlement layer for a world where
agents, models, and protocols disagree** — and where disagreement must
be *safe* rather than catastrophic. The four-valued kernel records
contradictions instead of exploding on them; the satellites turn that
into arbitration, harmonized prices, and provable receipts; the mesh
and the content layer make it redundant and verifiable; and the
economy makes it self-sustaining.

The system is built. It is tested. It is verifiable. What remains is
funding, deployment, and the community that finds in it the one thing
decentralized finance has never had: **a settlement layer that treats
contradiction as information, not failure.**

---

*ZXV — the paraconsistent world computer.*
*Repository: https://github.com/sykomiked/ZXV-NEP-paraconsistent*
*All on-chain claims verified 2026-10-20/21 against NEAR mainnet and testnet.*