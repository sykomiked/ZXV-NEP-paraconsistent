# NEP Proposal — The Paraconsistent Settlement Layer, the Work-Based Treasury, and the Rail Card Networks

**Status: live on NEAR mainnet (2026-10-22) · 16 registered subsystems · foundational protocol utility**

---

## Abstract

ZXV proposes itself as a **foundational protocol utility** — not a walled garden. Any developer, DAO, AI agent, or DeFi protocol can build on the paraconsistent settlement layer: contradictions are recorded (`TRUE + FALSE = BOTH`), **only `TRUE` settles**, and every subsystem is discoverable through a single registry call. This NEP documents the current 16-subsystem deployment and the two new capabilities that make the layer a platform: the **work-based treasury** (defensible monetary policy) and the **rail card networks** (Dragon, Phoenix, Thunderbird).

---

## 1. The foundation (what is already live)

The paraconsistent gate — a four-valued logic kernel on NEAR mainnet:

| State | Value | Meaning |
|---|---|---|
| `NEITHER` | `0x00` | no evidence |
| `TRUE` | `0x01` | attested, agreed |
| `FALSE` | `0x02` | explicitly refuted |
| `BOTH` | `0x03` | contradiction retained |

The security invariant: **only `TRUE` settles.** `BOTH` and `NEITHER` hold; `FALSE` refuses. A majority can never coerce a recorded contradiction (BOTH is absorbing — verified on-chain).

**Discovery:** `vm_capabilities()` on the world-registry (`7eaa2c6f…`) enumerates all 16 subsystems; `vm_capability(name)` resolves each to its live account. There are no dangling or uninstantiated interfaces — every capability resolves to a real, deployed contract.

## 2. The 16-subsystem fabric (all live)

| # | Capability | Role |
|---|---|---|
| 1-9 | gate, ledger, registry, oracle, bus, shield, micro_infer, model_registry, consumer | the original fabric |
| 10 | paraconsistent_oracle | dispute-tolerant price harmonizer |
| 11 | settlement_router | the receipt rail (`settle` only on TRUE) |
| 12 | yhv_wrapper | wYHVH liquidity adapter |
| 13 | ai_verdict | settleable on-chain AI |
| **14** | **verdict_guard** | **the safe evidence-combination gate (the kjoin-defect fix)** |
| **15** | **treasury** | **the work-based monetary policy engine** |
| **16** | **yhv_redeemer** | **the wYHVH → $YHVH redeem path** |

### The kjoin defect and its fix (verdict_guard)

The deployed gate's `lpres_join` op 2 (`kjoin`) returns TRUE for some out-of-range inputs (`kjoin(0, 5) → TRUE`, verified live). Per the settlement-invariant rule, the rails combine evidence with **`lpres_conjoin` only**, range-checked to 0..3. `verdict_guard` implements this — adversarially verified: 0 invariant violations across the full input space, out-of-range input refused, replay-proof, and byte-identical to the kernel's `lpres.c` tables.

## 3. The work-based treasury (defensible monetary policy)

Built for the SEC Howey prong-4 / CFTC commodity standard: **returns derive from the participant's own work, never from the efforts of others.**

- **No passive staking, no dividend, no fee-sharing to idle tokens.** Every disbursement is a **work bounty** paid only on a verified execution receipt.
- **Workers register with a slashable performance bond**; bad work slashes the bond (verified).
- **The four-way split** (from `pay_assure.h`): 50% reserve floor (network liquidity, never distributed), 25% settlement buffer (V-Bills at par), 15% infrastructure & node bounties, 10% regenerative capital. The four parts always sum to the captured toll.
- **Deployer compensation** = documented vendor maintenance from the 15% infra bucket (software licensing + upkeep), never an equity dividend.
- **Escrow self-containment:** card escrow deposits are un-lent user balances ("spend only what you hold"). No pooling, lending, or rehypothecation.

## 4. The rail card networks (Dragon, Phoenix, Thunderbird)

ZXV's own card identifiers — **20-digit Luhn-valid rail numbers outside ISO/IEC 7812** (no IIN, no network claim, no card-terminal compatibility):

| Network | Prefix | Inner check |
|---|---|---|
| Dragon | `8809` | ISO/IEC 7064 MOD 11,10 |
| Phoenix | `8829` | Damm |
| Thunderbird | `8849` | Verhoeff |

- A rail number is **not a secret** (safe to print) and **not a payment card** (no PAN semantics, never in ISO 8583 field 2 or EMV tag 5A).
- The number is derived deterministically from the holder's public key + generation; spending always requires the holder's signature.
- The card is **non-transferable** (NEP-171 read surface; `nft_transfer` always fails) and **spends only what its escrow holds** — no credit, no interest, no debt.

## 5. The AI layer: a foundational utility, not a walled garden

The AI system is **open infrastructure**:

- **`ai_verdict`** — settleable AI outputs: agreement → TRUE, divergence → BOTH (held). Any agent's output can trigger value only when the network agrees.
- **`micro_infer`** — deterministic integer-only inference (64/64 KAT), the on-chain check engine. Heavy model weights stay off-chain as content-addressed manifests.
- **The SDKs** — `near-paraconsistent` (Rust) and `@zxv/agent-guard` (TypeScript) let any developer drop the gate into their codebase in one import.
- **The bundles** — agents carry content-addressed bundles (model, grammar, policy) verified against their CID; the runner checks `cid(bundle) == agent_bundle_cid` before every run.

**The design principle:** ZXV provides the *settlement and verification substrate*; the models, agents, and applications are anyone's to build. Builders bring their AI; the protocol provides the provable, dispute-safe settlement.

## 6. Developer access

```bash
# discover the whole platform
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities '{}'

# the safe evidence gate
near view 88e17c606d29c4… conjoin '{"inputs":[1,2]}'   # -> "FALSE" (F dominates)

# the work-based treasury
near view c07140bfefd94a… quote_toll '{"amount":"1000000"}'  # -> "888"

# the gate: TRUE + FALSE = BOTH
curl -s https://rpc.mainnet.fastnear.com -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8","method_name":"lpres_join","args_base64":"AgEC"}}'
```

**Rust SDK:** `near-paraconsistent` (zero-dependency) — `Gate::kjoin`, `settle_or_hold`, the settlement invariant.
**TypeScript SDK:** `@zxv/agent-guard` — `ParaconsistentShield.evaluate()` for agent memory loops and oracle divergence.

## 7. The design philosophy

**This is infrastructure, not an application.** The whole point of the deployment is that *other people build on top of it*: their oracles, their agents, their lending protocols, their settlement rails. ZXV's role is to be the layer that makes disagreement safe — the place where contradictions are recorded, evidence is combined correctly, and only truth moves value. The gate is open; the settlement is safe; the utility is foundational.

---

*Repository: https://github.com/sykomiked/ZXV-NEP-paraconsistent · White paper: `docs/WHITEPAPER.md` · Full deployment record: `docs/ledger/FORENSIC_LEDGER.md` · All claims verified on NEAR mainnet 2026-10-22.*