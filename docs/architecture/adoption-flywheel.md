# The ZXV OS on NEAR — compatibility layers & the adoption flywheel

**How to build the best custom system, as efficiently as possible, and
make it so useful that automated systems find and use it with zero
marketing.**

## 0. The hard constraint (shapes everything)

On NEAR, deployed contract code is **public and readable by anyone**.
"Not sharing the files" cannot hide the implementation — the wasm is on
the chain. The moat is therefore **never secrecy**. It is:

1. **State** — the ledger, the proofs, the settled history (not
   copyable).
2. **Network position** — being the registered, discoverable, indexed
   default.
3. **Proofs** — the on-chain evidence (48-cell KAT, verified receipts)
   that competitors would have to rebuild from zero.
4. **Being the dependency** — if other systems must call us to
   function, adoption is automatic.

The strategy: **share the interface (that is what adaptation needs);
keep the advantage in the state and the proofs (that is what cannot be
copied).**

## 1. The compatibility matrix — which standard surface makes WHICH automated system find us

| Standard | What scans it | Status |
|---|---|---|
| **NEP-141** (`ft_metadata`, `ft_total_supply`, `ft_balance_of`, `ft_transfer*`) | wallets, indexers, DEXes, portfolio trackers | ✅ live |
| **NEP-145** (`storage_deposit`, `storage_balance_of`) | DEXes, any contract that must register accounts | ✅ live |
| **NEP-297** (standard event logs) | indexers, explorers, analytics, watchdogs | ✅ live |
| **NEP-141 `reference` field** (→ the release root CID) | indexers that read token metadata for off-chain data | ✅ live (our discovery seed) |
| **Ref Finance pool** | DEX aggregators, price oracles, portfolio tools | ✅ pool #8975 exists |
| **CISEX index / armada doors** | our own mesh + any IPFS resolver | ✅ live |
| **The NEP** (governance surface) | NEAR ecosystem tooling, the working group | ✅ repo live |
| **`vm_info` / `vm_capability`** (the VM discovery surface) | any agent that probes for the OS | ✅ authored (deploys with satellites) |

**The principle:** every automated system that already scans NEAR for
standards finds us through at least one row. The kernel is already
visible to every NEP-141 scanner on the network — that is the
zero-marketing entry point.

## 2. The layer stack (the bridge from OS image to utility)

```
┌──────────────────────────────────────────────────────────────┐
│  L5  ADAPTERS (what other ecosystems poll)                   │
│      Ref pool · IPFS/armada doors · CISEX · future bridges   │
├──────────────────────────────────────────────────────────────┤
│  L4  STANDARD COMPATIBILITY (indexers auto-scan this)        │
│      NEP-141 · NEP-145 · NEP-297 · reference→CID             │
├──────────────────────────────────────────────────────────────┤
│  L3  DISCOVERY (self-describing, gas-free)                   │
│      vm_info · vm_capability · vm_capabilities · boot CID    │
├──────────────────────────────────────────────────────────────┤
│  L2  SUBSYSTEMS (utility providers, near-sdk-rs)             │
│      oracle-gateway · ipc-bus · execution-shield · registry  │
├──────────────────────────────────────────────────────────────┤
│  L1  KERNEL (live, 27 syscalls, the ALU + ledger)            │
│      lpres_join · vouchers · V-Bills · tributes · $YHVH      │
├──────────────────────────────────────────────────────────────┤
│  L0  BOOT IMAGE (IPFS)                                       │
│      boot manifest · release root · docs · NEP · proofs      │
└──────────────────────────────────────────────────────────────┘
```

The layers are the bridge: **L0-L1 are the OS image; L2-L3 make it
discoverable; L4 makes existing scanners find it; L5 makes other
ecosystems able to use it.** Nothing is shared beyond the interface —
the state (L1) and the proofs are the uncopyable advantage.

## 3. The adoption flywheel (how it spreads with zero marketing)

The mechanism is **dependency, not promotion**:

```
A protocol needs safe multi-agent settlement
  → probes vm_info() (gas-free)          [L3 discovery]
  → finds the gate (the only TRUE-only 4-valued arbiter on NEAR)
  → registers (NEP-145, free)            [L4 compatibility]
  → uses lpres_join for its disputes     [L1 the dependency]
  → its receipts/verdicts are on-chain   [L0 proofs]
  → the next protocol sees the activity and probes too
```

Why this works without marketing:

1. **The need is real and unmet.** No other NEAR system offers a
   four-valued, TRUE-only-settles arbitration surface. The protocol
   that hits agent/oracle divergence *needs* what only we have.
2. **Discovery is free and automatic.** `ft_metadata.reference` →
   release root → boot manifest → registry. Every NEP-141 indexer
   already sees the seed. `vm_info()` is one gas-free call.
3. **Adoption is a lookup, not a project.** The interface is standard
   patterns (NEP-141/145/297 + promises). The `status` field (live /
   authored) means no breaking change as satellites deploy.
4. **Every use compounds the proof.** Each verdict, receipt, and
   settlement is on-chain — visible activity that later protocols see
   and trust. The state grows more valuable the more it is used.
5. **The utility is the dependency.** We are not an app users choose;
   we are the primitive other systems must call to function. That is
   the only adoption that needs no marketing.

## 4. Custom vs. standard — the precise split

| What | Standard (share) | Custom (our advantage) |
|---|---|---|
| The token surface | NEP-141/145 — so indexers see us | the ledger behind it (vouchers, V-Bills, tributes) |
| The logic | the four states + kjoin — so contracts can call it | the state journal, the proofs, the dispute threads |
| Discovery | vm_info/vm_capability — so agents can adapt | the subsystems' internal state |
| Events | NEP-297 — so indexers log us | the audit chain, the receipts |
| The interface | public, documented, tested | the accumulated on-chain state and reputation |

**"Non-standard" is only an advantage where it creates a dependency
nobody else provides (the gate, the shield, the verdicts). Everywhere
automated systems scan for standards, we MUST be standard — that is the
price of being found.**

## 5. Efficiency (best system, least cost)

1. One kernel, one registry, one manifest — satellites are
   **registered**, not redeployed (5 contracts ≈ 6 NEAR staked,
   refundable; ~0.01 NEAR burned).
2. Gas-free reads everywhere (discovery, gate, ledger views) — probing
   and using costs nothing, so automation can afford to look and call.
3. Writes rare and deliberate (init, registration, ~0.008 NEAR
   anchors).
4. The 9-node armada mirrors the boot image regionally at no extra
   cost.
5. Reuse of the exact code that already runs the fleet kernel — no
   parallel implementation.

## 6. The one-page answer

- **Share the interface** (NEP-141/145/297 + vm_info) — that is what
  makes automated systems find and use us.
- **Keep the state and proofs** — that is the uncopyable moat.
- **Be the dependency** — the only TRUE-only 4-valued arbiter, the
  gated vault, the dispute oracle: protocols that need these must call
  us.
- **Let the standards do the marketing** — every NEP-141 scanner on
  NEAR already sees the kernel; the flywheel does the rest.