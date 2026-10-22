# NEP — The Paraconsistent Settlement Layer: a Foundational Protocol Utility on NEAR

**Category: Proposal · Status: Live on NEAR mainnet (verified 2026-10-22)**

---

## Abstract

ZXV is a paraconsistent settlement layer for NEAR: a four-valued logic
kernel in which **only `TRUE` settles**. Contradictions (`TRUE + FALSE =
BOTH`) are recorded, never coerced, never a panic. The system is
deployed as 18 registered, discoverable subsystems plus a consolidated
core, and it ships a developer kit (Rust crate + TypeScript SDK) so any
protocol, DAO, or AI agent can integrate in one import.

This NEP proposes the layer as a **foundational protocol utility** — open
infrastructure others build on, not a walled garden.

---

## 1. The problem ZXV solves

Decentralized systems fail at the boundary of disagreement:

| Failure mode | Classical behavior | ZXV behavior |
|---|---|---|
| Two oracle feeds diverge | revert / deadlock / unfair liquidation | **held as `BOTH`** — no value moves |
| Two AI models disagree | panic or pick a side | **held** until evidence resolves |
| Two agents assert conflicting directives | undefined behavior | **recorded, never coerced** |
| Settlement ambiguity | the transaction blocks | **held; only `TRUE` settles** |

The principle of explosion means one contradictory input can destroy a
whole transaction. ZXV replaces that: contradictions are *information*,
and the gate makes disagreement safe by construction.

## 2. The four-valued foundation

| State | Value | Meaning |
|---|---|---|
| `NEITHER` | `0x00` | no evidence yet |
| `TRUE` | `0x01` | attested, agreed |
| `FALSE` | `0x02` | explicitly refuted |
| `BOTH` | `0x03` | contradiction retained |

The knowledge join: `kjoin(a, b) = (a | b) & 3`. The settlement
invariant: **only `TRUE` is settleable**; `BOTH` and `NEITHER` hold;
`FALSE` refuses. `BOTH` is absorbing — a majority can never coerce a
recorded contradiction (verified on-chain).

## 3. The deployed fabric (mainnet, verified)

`vm_capabilities()` on the world-registry
(`7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`)
returns 18 capabilities, each resolving to a live contract:

| Capability | Role |
|---|---|
| gate, ledger | the kernel: the gate + the $YHVH ledger |
| registry | the boot sector (discovery) |
| oracle, bus, shield | the original fabric |
| micro_infer, model_registry | deterministic on-chain AI + model manifests |
| consumer | the cross-contract integration proof |
| paraconsistent_oracle | dispute-tolerant price harmonizer |
| settlement_router | the receipt rail |
| yhv_wrapper | wYHVH liquidity adapter |
| ai_verdict | settleable on-chain AI |
| **zxv-core** | the consolidated core hosting five capabilities: |

`zxv-core` (`f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`)
is one lean contract (149,974 bytes — 79% smaller than five separate
near-sdk contracts) hosting:

- **verdict_guard** — `conjoin`, `is_settleable`: the safe evidence
  combination (the kjoin-defect fix, range-checked `lpres_conjoin`)
- **treasury** — `capture_toll`, `quote_toll`, `register_worker`: the
  work-based economy (Howey prong-4 defensible: bounties per verified
  receipt, slashable worker bonds, no passive staking)
- **yhv_redeemer** — the wYHVH → $YHVH redeem path
- **rails_registry** — `verify_number`: the Dragon (`8809`, MOD 11,10),
  Phoenix (`8829`, Damm), Thunderbird (`8849`, Verhoeff) 20-digit rail
  card networks, check digits byte-for-byte from `cn_check.c`
- **rails_receipts** — `append`, `head`, `verify_chain`: the
  hash-chained settlement journal

## 4. The developer kit (the "not a walled garden" proof)

**New repository: `https://github.com/sykomiked/zxv-developer-kit`**

- **`zxv-sdk`** (Rust crate, `no_std`, zero-dependency core):
  `Gate::conjoin`, `LogicState`, `settle_or_hold`, the rail check
  digits — crates.io-ready, 6 tests pass
- **`@zxv/developer-kit`** (TypeScript SDK): `discover()`,
  `verifyCard()`, `oraclePrice()`, `isSettleable()`, `chainHead()` —
  verified against live mainnet (18 subsystems)

```rust
use zxv_sdk::{Gate, LogicState, settle_or_hold};
let verdict = Gate::conjoin(LogicState::True, LogicState::False);
assert!(!verdict.is_settleable());   // contradiction held, no panic
```

```ts
import { Zxv, LogicState } from "@zxv/developer-kit";
const zxv = new Zxv();
const caps = await zxv.discover();                  // 18 subsystems
await zxv.verifyCard("88090016741821469134");       // "dragon"
await zxv.isSettleable(LogicState.BOTH);            // false
```

## 5. Why it is a foundational utility

1. **Discovery is the boot contract** — any protocol probing
   `vm_capabilities()` finds the whole topology; no interface is
   dangling.
2. **The settlement invariant is the moat** — only `TRUE` moves value;
   a divergent price or a conflicting AI output can never trigger an
   unfair payout or liquidation.
3. **The AI layer is open** — `ai_verdict` and `micro_infer` provide the
   *verification substrate*; the models, agents, and applications are
   anyone's to build.
4. **The economy pays for work, not holding** — the treasury's bounties
   are earned per verified receipt; idle capital earns nothing
   (defensible under the SEC's Howey prong 4 and CFTC commodity
   standards).
5. **The cards are ZXV's own identifiers** — 20-digit Luhn-valid rail
   numbers outside ISO/IEC 7812 (no IIN, no card-network claim, no
   PAN semantics).

## 6. Verification (gas-free views)

```bash
# the whole platform
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities '{}'

# the gate: TRUE + FALSE = BOTH
curl -s https://rpc.mainnet.fastnear.com -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8","method_name":"lpres_join","args_base64":"AgEC"}}'

# the safe evidence gate
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf conjoin '{"inputs":[1,2]}'

# the toll
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf quote_toll '{"amount":"1000000"}'

# the card networks
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf verify_number '{"number":"88090016741821469134"}'
```

## 7. Documentation

- White paper: https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/WHITEPAPER.md
- Developer kit: https://github.com/sykomiked/zxv-developer-kit
- Full deployment record: https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/ledger/FORENSIC_LEDGER.md
- Oracle-divergence case study: https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/CASE_STUDY_ORACLE_DIVERGENCE.md

## 8. Design philosophy

**This is infrastructure, not an application.** The point of the
deployment is that other people build on top of it — their oracles,
their agents, their lending protocols, their settlement rails. ZXV's
role is the layer that makes disagreement safe: where contradictions are
recorded, evidence is combined correctly, and only truth moves value.
The gate is open; the settlement is safe; the utility is foundational.

---

*Repository: https://github.com/sykomiked/ZXV-NEP-paraconsistent · Developer kit: https://github.com/sykomiked/zxv-developer-kit · All on-chain claims verified against NEAR mainnet 2026-10-22.*