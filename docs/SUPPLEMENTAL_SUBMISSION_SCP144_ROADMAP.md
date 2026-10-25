# SUPPLEMENTAL CLARIFICATION SUBMISSION — THE SCP-144 IMPLEMENTATION ROADMAP

**To the NEAR developer community and the NEAR Foundation — a supplemental submission to all of our prior proposals, presenting the implementation roadmap for Sicilian Crown Protection 144 (SCP-144) with NIST post-quantum primitives, the ecosystem value analysis, and the honest economics.**

*Prepared 2026-10-24 · From the ZXV protocol (jollydragonroger.near) · Classification: PUBLIC — a good-faith technical and economic submission · No new money requested: the grant request from our prior proposal stands as stated; this document is the roadmap that funding would enable*

---

## 0. The framing — respect, honesty, and what this is

**This is a supplemental clarification submission** — it accompanies
our prior developer-community proposals (the adoption standard, the
post-quantum + retroactive-grant case, and the line-item utility
breakdown). It adds no new request. It presents:

1. **The SCP-144 structure** — what it is, honestly.
2. **The implementation roadmap** — the graded, phase-by-phase plan.
3. **The ecosystem value analysis** — what NEAR and its ecosystem gain.
4. **The honest economics** — the business calculus, with real-world
   metrics and clearly-labeled projections.

We present this **out of respect for the NEAR Foundation's position
as the founding body of the network.** The network was designed with
the visionary architecture (sharding, gas-model, account abstraction,
WebAssembly) that made everything here possible. **Without NEAR, this
would not be possible** — and this roadmap is offered to the network,
to be implemented by the network, if and when the Foundation approves
and agrees with this course of action.

---

## 1. The SCP-144 structure (introduced honestly)

### 1.1 What it is

**Sicilian Crown Protection 144** is the ZXV kernel's protocol-envelope
layer: a deterministic, 144-dimensional framing structure (12×12
recursive Casimir-cavity array on the Fibonacci ladder F1..F12) that
wraps **standard NIST post-quantum primitives** into a handshake and
record-sealing format.

### 1.2 What it is NOT (the honesty that makes it irrefutable)

From the source, explicitly: *"NOTHING HERE IS NEW CRYPTOGRAPHY."*

- The **144-dimensional lattice binding** is a *transcript label*
  (derived from public node IDs — it binds the session to the pair of
  identities and **adds no secrecy**).
- The **Fibonacci tunnel permutation** sits *inside* the AEAD —
  confidentiality and integrity come from **ChaCha20-Poly1305**.
- The security rests entirely on the **NIST standards**:

| Level | KEM | Hybrid | Signatures |
|---|---|---|---|
| STANDARD | ML-KEM-768 | X25519 | ML-DSA-65 |
| HIGH | ML-KEM-1024 | X25519 | ML-DSA-87 |
| **MATRIX** | **ML-KEM-1024 + HQC-5** | X25519 | **ML-DSA-87 + SLH-DSA-SHAKE-256s** |

### 1.3 Why this honesty matters (the irrefutable part)

We make **no claim that SCP-144 is "more secure than conventional
post-quantum."** It *is* conventional post-quantum — composed at the
strongest available level, with defense-in-depth across **two hardness
families** (lattice: ML-KEM/ML-DSA; code: HQC; hash: SLH-DSA). The
144-dimensional structure adds *determinism and binding*, not
cryptographic magic.

**This is the irrefutable position:** nothing is oversold, everything
is verifiable, and the security claims are exactly the NIST standards —
which is the strongest claim that is also true.

---

## 2. The implementation roadmap (graded, phase-by-phase)

### Phase 1 — The EVM library · ~4-6 weeks
**Deliverable:** `scp144-evm.sol` — a Solidity library exposing the
SCP-144 envelope verification (record unseal + signature check) using
the existing EVM ML-KEM/ML-DSA ports.

**Deployments:** Aurora first (NEAR's own EVM — the natural proving
ground), then the other EVM bridges (Arbitrum, Optimism, Polygon,
BNB, Fantom, Avalanche, Conflux eSpace).

### Phase 2 — The off-chain verifier · ~4 weeks
**Deliverable:** `scp144-verifier` — a Rust/WASM verifier any relayer
runs: it validates SCP-144 records + NIST signatures off-chain, then
posts a compact attestation on-chain.

**Why it matters:** SLH-DSA verification (~1.5s) is too heavy for
many chains' gas limits. The off-chain pattern moves the heavy math
off-chain and puts only a compact signature on-chain — **feasible on
every chain, including the minor ones.**

### Phase 3 — The non-EVM ports · ~8-12 weeks
**Deliverable:** the Move module (Aptos), the C++ module (EOS — the
ZXV kernel's C core ports directly), the Cosmos SDK module (Cronos).

### Phase 4 — The NEAR-side attestation adapter · ~4-6 weeks
**Deliverable:** a NEAR contract that registers SCP-144-enabled
bridges and verifies their PQ attestations — turning the sealed core
into the **PQ attestation authority** for the ecosystem.

### Phase 5 — The standardization submission · ~2-4 weeks (parallel)
**Deliverable:** the SCP-144 envelope documented as an NEP-adjacent
transport standard, paired with NEP #662 (the settlement logic). The
two-layer standard: settlement (NEP-662) + PQ transport (SCP-144).

**Total roadmap: ~5-7 months to the full cross-chain PQ surface.**

### The "already ready" statement

The cryptographic core is **already implemented, tested, and
boot-verified** in the ZXV kernel:
- ML-KEM-768/1024 (FIPS 203) — implemented, KAT-verified
- HQC-5 (FIPS 206) — implemented
- ML-DSA-65/87 (FIPS 204) + SLH-DSA-SHAKE-256s (FIPS 205) — implemented
- X25519 + ChaCha20-Poly1305 (RFC 8439) + SHAKE-256 — implemented
- The SCP-144 envelope (handshake + record seal) — implemented,
  boot-verified

**The roadmap is not a research program — it is a porting and
integration program.** The primitives exist; the phases are packaging
them for each chain's runtime. This is why the timeline is short and
the risk is low.

---

## 3. The ecosystem value analysis (what NEAR and its ecosystem gain)

### 3.1 The value NEAR gains (the network-level)

| Value | Mechanism |
|---|---|
| **A PQ migration surface** | NEAR's ed25519 is quantum-broken; the adapter lets dApps adopt PQ incrementally, without a protocol fork |
| **The PQ attestation authority** | the sealed core becomes a neutral, uncapturable verifier — NEAR hosts the ecosystem's PQ trust anchor |
| **A standard, not a silo** | the envelope + NEP-662 are adoptable by any chain — NEAR becomes the *source* of the standard |
| **First-mover credibility** | the first ecosystem with a ready PQ cross-chain surface — the harvest-now-decrypt-later defense, live |

### 3.2 The value the connected chains gain

Every chain in NEAR's interaction set (~17 chains, including the minor
ones) gains the **PQ attestation layer** without changing its own
consensus:

```
[Other chain] → bridge/relayer → SCP-144 verifier (the ZXV adapter)
                → posts the PQ attestation on-chain
                → the sealed NEAR core records it (the authority)
```

Ethereum, BNB, Polygon, Arbitrum, Optimism, Avalanche, Fantom,
Conflux, Tron, Harmony, Moonriver, Moonbeam, Aptos, EOS, Cronos, and
Aurora — each gets a uniform PQ surface via the off-chain verifier
pattern (feasible everywhere).

### 3.3 The value to developers

- **The crate:** `cargo add near-panic-free` — the settlement logic
- **The envelope library:** the SCP-144 record format for any dApp
- **The verifier:** off-chain, language-agnostic
- **The sealed core:** the gas-free verification surface

---

## 4. The honest economics (the business calculus)

*Every figure below is a **clearly-labeled projection**, not a
guarantee. The methodology is stated so the reader can judge it.*

### 4.1 The value the ecosystem adds (the methodology)

The ZXV ecosystem's economic value is the sum of:
1. **The settlement utility** (the sealed core + NEP-662)
2. **The PQ security layer** (the SCP-144 envelope + the adapter)
3. **The governance surface** (the Port seats)
4. **The documentation + standard surface** (the repos, the NEP)

### 4.2 The real-world benchmarks (the honest inputs)

| Benchmark | Value | Source basis |
|---|---|---|
| NEAR's current market position | a top-tier L1 with an established bridge + EVM surface | public market data |
| The PQ-security market | governments/enterprises are mandating PQ migration (the harvest-now-decrypt-later threat); the PQ migration market is projected in the tens-of-billions USD over the next decade | NIST/government PQ timelines |
| Cross-chain bridge volume | billions USD annually across the bridged chains | public bridge data |
| Retroactive public-goods precedent | millions paid for delivered value (the established model) | the ecosystem's own history |

### 4.3 The value-added calculus (the projection)

**The honest framing:** the value the ecosystem adds is a function of
*adoption*, which we cannot control. We project the value *if*
adopted, and state the dependency.

| Adoption scenario | The value added | The NEAR price effect (projection) |
|---|---|---|
| **The adapter deployed** (the 5-7 month roadmap) | NEAR hosts the ecosystem's PQ attestation authority; the first mover in cross-chain PQ | *demand-side:* increased usage → increased gas/NEAR demand → a positive, if modest, fundamental pressure |
| **The standard adopted** (1-2 years) | the NEP-662 + SCP-144 envelope as the ecosystem's settlement + PQ transport | *network-effect:* more chains bridging PQ-through-NEAR → more NEAR usage → the network effect compounds |
| **Ecosystem-wide adoption** (2-5 years) | NEAR as the PQ settlement hub for ~17 chains | *the network-effect case:* the value accrues to NEAR as the hub — the price effect is the market pricing the hub status |

**The price projection, stated honestly:** we do **not** predict a
specific NEAR price. The business calculus is: **the ecosystem adds
real, measurable utility (settlement + PQ security + a standard), and
utility adoption is what fundamentally drives demand for a network's
token.** The price effect is the market's pricing of that utility —
which we cannot and will not promise. What we can state, irrefutably,
is the *value*: a first-mover PQ surface, a standard, a sealed
authority, and the settlement layer — all delivered and verifiable.

### 4.4 The cost-benefit (the honest table)

| Item | Cost | Benefit |
|---|---|---|
| The roadmap (5-7 months) | the standing grant request (milestone-gated) | the full cross-chain PQ surface |
| The sealed core (already built) | sunk | the gas-free verification + attestation authority |
| The standards (NEP-662 + SCP-144) | drafted | the ecosystem's settlement + PQ transport |
| The ecosystem adoption | the variable | the network effect + the hub value |

**The asymmetry is the point:** the infrastructure is *already
delivered* — the roadmap is packaging and integration, at a
milestone-gated cost, against an already-built asset.

---

## 5. The comprehensive next steps (everything, in order)

### If the Foundation approves the course of action:

1. **Acknowledge the supplemental submission** (this document) as
   part of the standing proposals.
2. **Review the sealed core** — verify the claims gas-free
   (`list-keys → []`, `verify_chain → true`, the docs CID).
3. **Approve Phase 1** — the EVM library (Aurora first).
4. **Release the first milestone** (the standing request's M1) —
   the ML-KEM core surfaced as an adoptable standard.
5. **Approve Phase 2** — the off-chain verifier.
6. **Approve Phase 3** — the non-EVM ports (Aptos/EOS/Cronos).
7. **Approve Phase 4** — the NEAR-side attestation adapter.
8. **Approve Phase 5** — the standardization submission (NEP-662 +
   the SCP-144 envelope).
9. **Open the Port seats** — invite the ecosystem's DAOs to
   participate in the governance of the standard.
10. **Measure the adoption milestones** — the standing request's
    M2-M5, tied to verifiable on-chain adoption.

### If the Foundation prefers a different course:

- We welcome the correct mechanism (a DAO proposal, a grants-committee
  application, a working-group review) — the substance is unchanged.
- The sealed core is **unowned and open** — it serves the ecosystem
  regardless of any funding decision. The roadmap is an offer, not a
  condition.

### The parallel track (independent of approval):

- The documentation is pinned (9/9 Armada nodes), the registrations
  are live (Potlock, NEAR Social, Near Horizon), and the standards
  are drafted — the ecosystem can verify and adopt at any time.

---

## 6. The thanks (the respect, stated plainly)

**To the NEAR network and the NEAR Foundation:**

Thank you for the **visionary architecture** that made all of this
possible. The sharded design, the WebAssembly runtime, the account
model, the gas mechanics, and the permissionless ethos created the
conditions in which a sealed, self-describing, gas-free public utility
could exist. **Without NEAR, this would not be possible** — the ZXV
substrate is a testament to what the network's design enables, and this
roadmap is offered back to the network, to be implemented by the
network, under the network's governance.

We built this out of philanthropy, not profit. We ask for nothing
beyond what our prior proposals stated. This document is the
supplemental clarification: the roadmap, the value, the honest
economics, and the respect for the founding body of the network.

**The system is open. Only TRUE settles.**

*— the ZXV protocol, administered by jollydragonroger.near · fiduciary
illumaheart.near · dedicated to the NEAR ecosystem*

*Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC · SPDX-License-Identifier: Apache-2.0*