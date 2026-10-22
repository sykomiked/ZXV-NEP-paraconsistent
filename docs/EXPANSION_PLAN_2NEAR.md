# The 2-NEAR Kernel-Feature Expansion Plan

**Prepared 2026-10-22 · testnet-first · within the incoming ~2 NEAR deposit**

---

## 1. The budget

| Source | Amount |
|---|---|
| The 2 NEAR deposit (to the primary) | +2.000 |
| The dfe804… balance return | +~2.08 (minus the 0.333 note) |
| **Deployable budget (primary's free after the sweep floor)** | **~2.4 NEAR** |
| Reserve for gas + the satellites' floors | ~0.5 NEAR |
| **Budget for the expansion** | **~1.9 NEAR** |

## 2. The goal: every kernel feature functional on-chain

The kernel's 261 modules reduce to **10 value-bearing systems**. The
architecture: one micro contract (`zxv-core`) hosting them as **feature
kits** — no new contracts per system, one shared runtime, one storage
payment. The expansion adds the missing kits to the existing micro
contract.

| Kernel system | Kit | Status |
|---|---|---|
| lpres (the gate) | `gate` | ✅ live |
| pay (the toll) | `treasury` | ✅ live |
| cardnet (the rails) | `rails` | ✅ live |
| vino (the economy) | `vino` | ✅ live |
| tribute (the Crown) | `tribute` | ✅ live |
| surplus (reserve math) | `surplus` | ✅ live |
| chronicle (the journal) | `chronicle` | ✅ live |
| allotment (the ballot) | `allotment` | ✅ live |
| broker (market intents) | `broker` | ✅ live |
| **rmag (exact rational money)** | **`rmag`** | ⏳ the next kit |
| **scp144 (the lattice bind)** | **`scp144`** | ⏳ view mirror |
| **aipi (remote inference escrow)** | **`aipi`** | ⏳ the compute kit |
| **chiglet (the agent)** | **`chiglet`** | ⏳ the agent kit |

## 3. The expansion sequence (testnet first)

### Phase 0 — testnet (free)
1. Add the **`rmag` kit** — exact rational money (reduce, compare, quote
   with no float; mirrors `kernel/src/rmag`). Pure views + the settlement
   router's amount math.
2. Add the **`scp144` kit** — the lattice bind digest view (mirrors the
   kernel's `scp144_bind`; the session-layer identity).
3. Add the **`aipi` kit** — the remote-inference escrow surface
   (`paid + refunded = reserved`, the QUORUM invariant as a view).
4. Add the **`chiglet` kit** — the agent-DNA identity (the `chg_kin`
   digest: owner key, lineage, parent — the agent's on-chain identity
   record).
5. **Audit on testnet**: every kit adversarially tested (the rmag math
   exhaustively; the aipi escrow invariant; the chiglet digest
   determinism).

### Phase 1 — mainnet (the ~1.9 NEAR)
6. Redeploy `zxv-core` with the 4 new kits. The wasm grows ~15-20 KB →
   ~0.15-0.20 NEAR extra storage (fits the existing account's free + the
   deposit).
7. Register the new kit names in the registry (`kits()` now returns 14).
8. **Mainnet audit**: all 14 kits verified live.

### Phase 2 — the reserve (if budget remains)
9. The agent **runner** surface (the Chiglet bundle CID check) — as a
   `chiglet` kit extension, no new contract.
10. The **payment-request format** (`zxv-pay:` URI) — an SDK + docs
    deliverable, zero storage.

## 4. The audit checklist (zero-fault standard)

| Kit | Attack tested |
|---|---|
| `rmag` | float-free exactness; overflow; reduce-to-lowest; the 4-state range |
| `scp144` | determinism; the bind digest matches the kernel's `scp144_bind` |
| `aipi` | `paid + refunded = reserved` invariant; replay refusal; underflow |
| `chiglet` | digest determinism; lineage binding; parent linkage |
| all kits | out-of-range refusal; no state-shape change (the struct-compat rule); every account solvent after |

## 5. The honest constraints

- **No new contracts** — the micro-contract architecture keeps every
  expansion at kit level (one storage payment, one runtime).
- **The struct never changes** — new kits use the existing `store` map
  under reserved ranges (the rule that prevented the deserialization
  bug).
- **Testnet proves it before mainnet** — the 4 kits are built and tested
  free; the mainnet redeploy is the only funded step.
- **The reserve (~0.5 NEAR)** stays for gas + the satellites' floors.

## 6. The deliverable state after

```
zxv-core (f18889ea…) — 14 feature kits
  gate · treasury · rails · receipts · vino · tribute · surplus ·
  chronicle · allotment · broker · rmag · scp144 · aipi · chiglet
```
Every value-bearing kernel system is functional on-chain, in one micro
contract, within the 2-NEAR budget, testnet-proven first.