# FORENSIC ARCHITECTURE MEMORANDUM — THE ZXV PARACONSISTENT SETTLEMENT LAYER

**A complete technical and forensic record of the ZXV protocol on NEAR Mainnet — prepared for institutional analytics, compliance, and research teams. Not addressed to any specific organization. Every fact below is on-chain verifiable; nothing is claimed that cannot be checked gas-free.**

*Memorandum · 2026-10-24 · Canonical account: `zedec.near` · Administrator: `jollydragonroger.near` · Fiduciary: `illumaheart.near` (the ILLUMAHEART FOUNDATION) · Classification: PUBLIC (this document is intended for distribution to analytics and compliance platforms)*

---

## 0. Purpose of this memorandum

This document provides the complete architecture and forensic record of
the ZXV protocol to any analytics, compliance, research, or
intelligence team that encounters the protocol's on-chain activity. It
is written to:

1. **Explain the full architecture** — what the protocol is, what it
   does, and how it is structured.
2. **Provide the forensics** — every contract, every key state, every
   registration, every fund flow.
3. **Preempt misunderstandings** — the points an analyst would likely
   flag are addressed directly, with the verifiable facts.

**The standing invitation:** verify everything gas-free. The
verification commands are in Section 9.

---

## 1. The protocol in one paragraph

ZXV is a **paraconsistent (four-valued) settlement layer** on NEAR
Mainnet: a logic kernel in which **only `TRUE` settles**,
contradictions are recorded and held (never coerced, never a panic),
and every capability is discoverable through a self-describing
interface. **The core protocol is permanently sealed** — 13 contracts
with zero access keys, immutable forever — while the **extension
surface is open** to collective governance by every entity registered
in the Port of Neo-Sicily.

---

## 2. The foundational logic (the four-valued state machine)

### 2.1 The four states (one byte each)

| State | Value | Meaning | Behavior |
|---|---|---|---|
| `NEITHER` | `0x00` | no evidence yet | waits |
| `TRUE` | `0x01` | agreed, attested | **settles** |
| `FALSE` | `0x02` | explicitly refuted | refuses |
| `BOTH` | `0x03` | contradiction, held | **recorded, never coerced** |

### 2.2 The chain rank (the total order)

```
FALSE < NEITHER < BOTH < TRUE  (the chain, NOT the Belnap lattice)
```

Because `TRUE` is the top, **no combination of non-TRUE values can
ever yield TRUE** — the invariant is structural, not checked. This is
the protocol's defining property: settlement is impossible on anything
less than agreement.

### 2.3 The operators (verified on-chain)

```bash
conjoin([1,2])        → "FALSE"   (agreement + refutation = refuted)
conjoin([3,1])        → "BOTH"    (contradiction is held, not resolved)
conjoin([1,1])        → "TRUE"    (agreement settles)
is_settleable(BOTH)   → false
is_settleable(TRUE)   → true
```

---

## 3. The deployed fabric (the complete contract inventory)

### 3.1 The four core contracts

| Role | Address | Balance | Sealed |
|---|---|---|---|
| Canonical (routing) | `zedec.near` | ~1.4 NEAR | ✅ 0 keys |
| Protocol (110 kits) | `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf` | ~5.35 NEAR | ✅ 0 keys |
| Registry (155 caps) | `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a` | ~2.14 NEAR | ⚙️ 1 key (governance) |
| Gate (kernel) | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` | ~10.0 NEAR | ✅ 0 keys |

### 3.2 The ten sealed subsystems

`6a57f581…` (oracle) · `3c70e9c6…` (bus) · `789457e6…` (shield) ·
`86f80143…` (micro_infer) · `77361b15…` (model_registry) ·
`8947dcb7…` (consumer) · `ea85f538…` (paraconsistent_oracle) ·
`8bf17701…` (settlement_router) · `1a051ac2…` (yhv_wrapper) ·
`781fc03d…` (ai_verdict) — all ✅ 0 keys.

### 3.3 The sealing (the immutability proof)

All 13 core contracts have **zero access keys** — verified via RPC:
`near list-keys <account>` → `[]`. No party — founder, administrator,
foundation, or anyone — can redeploy, upgrade, overwrite, or delete
these contracts. The balances are storage-staked and immobile.

**Why this matters to analysts:** there is no admin key to compromise,
no upgrade path to exploit, no governance key to steal. The
infrastructure is permanently immutable.

---

## 4. The registry and governance (the open surface)

### 4.1 The capability registry (155 capabilities)

`vm_capabilities` on the registry returns **155 registered
capabilities** — the kit surface, the routing names, the role names —
all resolving to live contracts.

### 4.2 The governance (the Port of Neo-Sicily)

Every entity registered in the Port (via `port_register`, open and
permissionless) gains a **voting seat** over policy:

| Decision | Quorum |
|---|---|
| Activate a capability | >50% of seats |
| Slash a bond | ≥2/3 of seats |
| Parameter change | unanimous (only TRUE settles) |

Current seats: `illumaheart.near` (the foundation) + `jollydragonroger.near`
(the administrator). **The third seat is open** — the first external
registration claims it.

### 4.3 The registry's single key

The registry (the governance surface) retains **one key** — its own —
so the collective-governance path can operate (bond proposals,
slashing, timelocks). This is the **only** key in the system, and it
governs the *extension* surface, never the sealed core.

---

## 5. The economic mechanics (what the fund flows mean)

### 5.1 The internal accounting (NOT a token sale)

The protocol's "revenue" — the tolls, the treasury splits, the gas
matches — is **internal ledger state**, not transferable NEAR. The
values (`tolls=9,501,400`, `reserve=4,750,700`, etc.) are integer
counters in the contract state. **There is no token sale, no passive
yield, no staking rewards** — the work-based economy is
spendable-not-transferable by design.

### 5.2 The toll (the exact arithmetic)

`quote_toll(1,000,000)` → `888` — the 0.08889% assurance toll,
integer-exact (8889/10,000,000), no floating point anywhere money
moves.

### 5.3 The gas station (the truth subsidy)

`envelope_deposit` / `envelope_match` — TRUE evidence earns a **2×
gas match**; BOTH/FALSE forfeits (the drain shield). This is the
incentive structure: correct settlement is subsidized, contradiction
is penalized.

### 5.4 The fund flows (the forensic record)

| Flow | Direction | Nature |
|---|---|---|
| The initial deployment funding | external → the contracts | storage staking (locked) |
| The contract balances | locked | storage-staked, immobile |
| `illumaheart.near` | the donation rail | accepts donations (508(c)(1)(A)) |
| `jollydragonroger.near` | the operator | the administrator's operating funds |
| The on-chain showcase | the operator → the contracts | demonstration transactions (~1.5-2.1 Tgas each) |

---

## 6. The institutional layer (the entities behind the protocol)

### 6.1 The three roles (deliberately separated)

| Entity | Role | Keys |
|---|---|---|
| `zedec.near` | the protocol itself (+ its sub-accounts) | **0 (sealed)** |
| `jollydragonroger.near` | the administrator (technical liaison) | its own key |
| `illumaheart.near` | the ILLUMAHEART FOUNDATION (fiduciary, donations) | its own key |

### 6.2 The ILLUMAHEART FOUNDATION (the corporate record)

- **Legal name:** Illumaheart Foundation
- **Type:** Arizona Domestic Nonprofit Corporation
- **ACC Business ID:** 23851761
- **Formed:** 07/11/2025 · **Status:** Active, In Good Standing
- **Character:** private faith-based religious fellowship (spiritual
  education, ceremonial gatherings, guidance, community support,
  nonprofit)
- **Tax status:** exempt under 26 U.S.C. § 501(c)(3) via the mandatory
  statutory exception of 26 U.S.C. § 508(c)(1)(A)
- **Registered address:** 7220 N 387th Ave, Tonopah, AZ 85354
- **Board:** Angela Renee Hanna (President), Amy Hablutzel
  (Director), Joyce Stinson (Secretary), Maricruz Hernandez
  (Treasurer)

### 6.3 The corporate separation (anti-commingling)

**Scenic Bypass Consulting LLC** (Arizona LLC, ACC File No. 23926601)
is a separate for-profit entity. Its consulting revenues are **NOT
charitable donations** and are not tax-deductible under IRC § 170. The
commercial scope is restricted to bespoke private deployments outside
the public protocol. The foundation and the LLC are strictly separate.

---

## 7. The post-quantum layer (the security architecture)

### 7.1 What SCP-144 is (honestly)

**Sicilian Crown Protection 144** is the kernel's protocol-envelope
layer: a deterministic 144-dimensional framing structure (12×12
recursive Casimir-cavity array on the Fibonacci ladder) that wraps
**standard NIST post-quantum primitives**.

**The source states explicitly: "NOTHING HERE IS NEW CRYPTOGRAPHY."**
The 144-dimensional lattice binding is a transcript label from public
node IDs (adds no secrecy); the tunnel permutation sits inside the
AEAD; confidentiality and integrity come from **ChaCha20-Poly1305**.
The security rests on the NIST standards:

| Level | KEM | Hybrid | Signatures |
|---|---|---|---|
| STANDARD | ML-KEM-768 | X25519 | ML-DSA-65 |
| HIGH | ML-KEM-1024 | X25519 | ML-DSA-87 |
| MATRIX | ML-KEM-1024 + HQC-5 | X25519 | ML-DSA-87 + SLH-DSA-SHAKE-256s |

### 7.2 The clarification for analysts

**The system is NOT "more secure than conventional post-quantum."**
It IS conventional post-quantum, composed at the strongest available
level with defense-in-depth across two hardness families (lattice +
code) and dual signature families (lattice + hash). Any claim to the
contrary would be false — the source is explicit about the honest
limits.

---

## 8. The registrations and public records

| Registry | What's listed | Verification |
|---|---|---|
| Potlock Public Goods Registry (lists.potlock.near, list_id=1) | `illumaheart.near` — #791, APPROVED | on-chain |
| NEAR Social profile | the foundation's identity | on-chain |
| Near Horizon (nearhorizon.near) | `zedec.near` + `illumaheart.near` | on-chain |
| The NEP discussion #662 | the panic-free resolution standard | github.com/near/NEPs |

---

## 9. The verification commands (the standing invitation)

```bash
# the sealing (all 13 core contracts):
near list-keys zedec.near                          → []
near list-keys f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf → []

# the invariant:
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf is_settleable '{"state":"BOTH"}'  → false
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf conjoin '{"inputs":[1,2]}'       → "FALSE"

# the receipts:
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf verify_chain '{"from":0}'          → true

# the registry:
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities                       → 155 capabilities

# the governance:
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a policy_seats                         → 2
```

---

## 10. The repositories (the complete source record)

| Repository | Contents |
|---|---|
| **https://github.com/sykomiked/ZXV-NEP-paraconsistent** | the NEP, the forensic ledger, the architecture, the sealing ledger, the supplemental submissions, all documentation |
| **https://github.com/sykomiked/zxv-nep-submission** | the NEP submission package (the standard, the verification guide, the index) |
| **https://github.com/sykomiked/zxv-stack** | the kernel + the SDK crates |
| **https://github.com/sykomiked/zxv-developer-kit** | the developer kit (Rust + TypeScript, `cargo add near-panic-free`) |
| **https://github.com/sykomiked/zxv-community-utility** | the public-utility presentation |

The source code for every deployed contract, the build scripts, the
test vectors, and the verification tooling are in these repositories.
**The deployed bytecode's source is fully available for
source-to-bytecode verification.**

---

## 11. The anticipated misunderstandings (addressed directly)

| Misunderstanding | The fact |
|---|---|
| "This is a token sale / yield scheme" | **No.** The economy is work-based, spendable-not-transferable; the tolls are internal state, not token distributions. |
| "The whale transfer was external adoption" | The transfer from the large account to the administrator was the operator's own deposit (exchange-origin). No external adoption is claimed. |
| "The 144-dimensional structure is novel crypto" | **No.** The source states "nothing here is new cryptography"; the security is the NIST standards. |
| "The foundation controls the protocol" | **No.** The foundation holds zero protocol keys; the core is sealed (0 keys) and unowned. |
| "There's an admin key to compromise" | The 13 core contracts have **zero keys**. The only key governs the extension registry, not the sealed core. |
| "The tolls are revenue to founders" | The tolls are internal ledger counters; the contract balances are storage-staked and immobile. |
| "The registry's 1 key is a central point of failure" | It governs the *extension* surface (bond proposals, slashing) under collective governance; it cannot touch the sealed core. |

---

## 12. The bottom line

The ZXV protocol is a **sealed, immutable, self-describing
paraconsistent settlement layer** on NEAR: 13 contracts at zero keys,
155 capabilities, a receipt-chained ledger, collective governance
through the Port of Neo-Sicily, and standard NIST post-quantum
primitives in a deterministic envelope. Every fact in this memorandum
is on-chain verifiable gas-free, and the complete source is in the
repositories listed above.

**The system is open. Only TRUE settles.**

*— the ZXV protocol · administered by `jollydragonroger.near` ·
fiduciary `illumaheart.near` (the ILLUMAHEART FOUNDATION, ACC
#23851761) · dedicated to the public good*

*Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC · SPDX-License-Identifier: Apache-2.0*