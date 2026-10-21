# ZXV — Developer Access Pack

**The paraconsistent world computer is live. Here is everything you need to build on it.**

*NEAR mainnet · 13 registered subsystems · verified 2026-10-21*

---

## What this is

ZXV is a four-valued paraconsistent settlement layer for NEAR: contradictions are **recorded, never coerced** (`TRUE + FALSE = BOTH`), and **only `TRUE` settles**. It is deployed as 13 registered, discoverable contracts — plus two developer SDKs so you never touch raw byte math.

The use cases that matter to builders:

- **Oracles / DeFi** — divergent price feeds become a held `BOTH`, not a panic; no unfair liquidations, no cascading reverts
- **AI agents** — conflicting model outputs are contained; only agreed (`TRUE`) outputs can trigger value movement
- **Settlement** — provable receipts (`zxv-receipt-NNNNNN`) for any multi-source decision

---

## 1. Discover the whole stack (one call, zero config)

```bash
# every registered subsystem, in order:
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities '{}'

# resolve any subsystem to its live account:
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capability '{"name":"ai_verdict"}'
```

`vm_capabilities()` returns all 13: `gate, ledger, registry, oracle, bus, shield, micro_infer, model_registry, consumer, paraconsistent_oracle, settlement_router, yhv_wrapper, ai_verdict`.

## 2. The contracts you'll use

| Subsystem | Account (64-hex) | What it does |
|---|---|---|
| **gate** (the kernel) | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` | the four-valued ALU (`lpres_join`) |
| **paraconsistent_oracle** | `ea85f5381ceff17553fa3093a2d67a0e6cc4853abd7c19487ed7a33391acc2f7` | dispute-tolerant price harmonizer |
| **settlement_router** | `8bf177019402d3d843b361a04d3ed7e21de637bf5d252956a15208f7950effae` | the receipt rail (`settle` on TRUE) |
| **ai_verdict** | `781fc03d4dfb9b29d88df561157b11db6dc1e02368eae6deef7998d188ff966e` | settleable on-chain AI |
| **micro_infer** | `86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe` | deterministic integer-only inference (64/64 KAT) |
| **yhv_wrapper** | `1a051ac2465c9a04e532fe420e2efbd8cc12f9a1d9f2a350470782fd3255ecf1` | $YHVH → wYHVH liquidity adapter |

Full 13-account table: [the white paper, Appendix A](https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/WHITEPAPER.md).

## 3. The SDKs (no raw bytes needed)

**Rust — `near-paraconsistent`** (crates.io-ready, zero-dependency):

```rust
use near_paraconsistent::{Gate, LogicState, settle_or_hold};

let state = Gate::kjoin(LogicState::True, LogicState::False);
assert_eq!(state, LogicState::Both);          // contradiction held
if !state.is_settleable() { /* hold, no panic */ }

// oracle-divergence guard for your contract method:
let verdict = settle_or_hold(&[feed_a, feed_b])?;  // Err(BOTH) = held
```

**TypeScript — `@zxv/agent-guard`** (npm-ready, zero-dependency):

```ts
import { ParaconsistentShield } from "@zxv/agent-guard";

const shield = new ParaconsistentShield();
const caps = await shield.discover();               // 13 subsystems
const verdict = await shield.evaluate([modelA, modelB]);
if (verdict.canSettle()) {
  await contract.executeTrade(...);                 // only TRUE settles
} else {
  console.log("Divergence contained: held as BOTH (0x03)");
}
```

## 4. Verify it yourself (gas-free views)

```bash
# the gate: TRUE + FALSE = BOTH (the promoted on-chain proof)
curl -s https://rpc.mainnet.fastnear.com -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8","method_name":"lpres_join","args_base64":"AgEC"}}'
# -> 0x03 (BOTH)

# the settlement invariant: BOTH is not settleable
near view 8bf177019402d3d843b361a04d3ed7e21de637bf5d252956a15208f7950effae is_settleable '{"state":"BOTH"}'
# -> false

# deterministic on-chain AI
near view 86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe infer '{"token":3,"dim":4}'
# -> 3
```

## 5. Documentation & artifacts

| Item | Link / CID |
|---|---|
| **White paper** | [docs/WHITEPAPER.md](https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/WHITEPAPER.md) · IPFS `QmPiju3iX37GnWkUCVW72KysSSSFY3teAJHaMDgTEumCxQ` |
| **Oracle-divergence case study** | [docs/CASE_STUDY_ORACLE_DIVERGENCE.md](https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/CASE_STUDY_ORACLE_DIVERGENCE.md) |
| **Full deployment record** | [docs/ledger/FORENSIC_LEDGER.md](https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/ledger/FORENSIC_LEDGER.md) |
| **Live telemetry** | the RSS feeds (NEAR validation, OS telemetry, CID status) — in the repo's `docs/telemetry/` |
| **Repository** | https://github.com/sykomiked/ZXV-NEP-paraconsistent |

---

## The one-line pitch

> Divergence becomes a pause, not a panic. Contradiction becomes
> information, not failure. **Only `TRUE` settles** — and it's live on
> NEAR mainnet, discoverable, with the SDKs to drop in today.

*Audited 2026-10-21: 12/12 on-chain checks PASS (kernel gate + supply,
micro-infer determinism, 13-subsystem discovery, all four DeFi utility
surfaces).*