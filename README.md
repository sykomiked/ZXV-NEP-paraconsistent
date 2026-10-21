# ZXV-NEP-paraconsistent

NEAR Enhancement Proposal (NEP) — **Paraconsistent Logic Resolution for
Multi-Agent On-Chain State Machines**.

A standardized four-valued paraconsistent logic resolution matrix
(`lpres_join`) allowing autonomous agents and oracles to ingest
contradictory inputs (`TRUE`, `FALSE`, `BOTH`, `NEITHER`) without
triggering execution panics, while strictly gating value settlement to
verified `TRUE` states.

## The Show and Tell demonstration (ready to post)

`docs/demonstration/onchain-ai-demo.md` — a live, verifiable mainnet
subsystem: deterministic AI on NEAR (8 contracts, zero-gas reads,
64/64 KAT, the paraconsistent gate with the exact raw-bytes call).

## Live mesh telemetry (RSS/WebSub)

[![Live Mesh Telemetry](https://img.shields.io/badge/Armada_Telemetry-RSS_Feed-orange.svg)](https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/telemetry.xml)

The 9-node Armada runs the ZXV paraconsistent kernel (the same engine
deployed on NEAR) as a neuro-symbolic arbitration supervisor. Each
resolved state — TRUE+FALSE→BOTH, contradiction contained, only TRUE
executes — is syndicated via WebSub/RSS. The feed is well-formed RSS
2.0 with the WebSub hub ping, sub-microsecond kernel latencies, and
NEAR silicon verification links. Track live distributed conflict
reduction: `https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/telemetry.xml` (per-node:
`the same GitHub path per feed`).

## Live OS telemetry — all 9 servers, one feed

[![9-Server Live Telemetry](https://img.shields.io/badge/ZXV_Armada-9_servers_live-green.svg)](https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/live_telemetry.xml)

Livestreamed RSS: all 9 geographic nodes (IND, EU, JPN, US-c, AUS, UK,
US-e, US-w, SIN) running the ZXV kernel as their operating system, one
feed. Each `<item>` is a node's live state — status, kernel served, mesh
counters — with unique GUIDs and the WebSub hub ping. This is the OS
advertisement and the live telemetry proof of concept:
`https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/live_telemetry.xml`.

## NEAR network-validation log (RSS)

[![NEAR Validation Log](https://img.shields.io/badge/NEAR_Validation-RSS_Log-blue.svg)](https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/network_validation.xml)

The live log of zero-gas calls to the NEAR deployments — every
validation and its result, syndicated. Each `<item>` records one view
call: contract, method, args, and the chain's response (gate→BOTH,
supply→60185, ledger→booted, discovery→9 capabilities, inference→3,
KAT→1, cross→Both). This shows the deployments working, without the
impracticality/gas of writing it to the chain:
`https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/network_validation.xml`.

## Full-system telemetry (NEAR · Armada OS · IPFS) — RSS

[![Full System Telemetry](https://img.shields.io/badge/ZXV_Full_System-RSS_Feed-purple.svg)](https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/system_telemetry.xml)

One feed showing the whole OS working end to end: the NEAR
deployments (gate→BOTH, vm_info→9 capabilities), the 9-node Armada
running the kernel (all LIVE), and the content-addressed CIDs resolving
on IPFS. Registered with Ping-O-Matic, the WebSub hub, and the Internet
Archive (immutable snapshots). The showcase is discoverable, not hidden:
`https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/system_telemetry.xml`.

## Status (2026-10-20)

**LIVE ON NEAR MAINNET:** the audited kernel + 5 satellite contracts
deployed (world-registry, oracle, bus, shield, micro-infer), each on its
own verified implicit account. The world-registry `vm_info()` is the
discovery surface (8 capabilities). model-registry is pending ~0.5 NEAR
for its deploy. Full addresses in `reference/CONTRACT.md`.

## Status

**Stage 1-2: Pre-proposal + Draft.** Live proof-of-concept on NEAR
mainnet; formal NEP draft prepared; discussion thread to be opened in
`near/NEPs` (Stage 1) before the draft PR (Stage 2).

## Contents

| Path | What |
|---|---|
| `neps/nep-paraconsistent-gate.md` | the formal NEP document (official template structure) |
| `docs/pre-proposal-discussion.md` | the Stage-1 discussion text (paste into near/NEPs Discussions → Standards) |
| `docs/evidence/lpres_join_onchain_proof.json` | all 16 cells of the kjoin gate, verified live on mainnet |
| `reference/lpres_gate.c` | the standalone reference implementation (exact kernel gate) |
| `reference/test_lpres_gate.c` | the 66-check test: kjoin table == on-chain proof, security property |
| `reference/build.sh` | host test + wasm32 build (the deployed contract's target) |
| `reference/CONTRACT.md` | the live contract surface + kernel source pointers |
| `reference/lpres.rs` | the Rust crate (`LogicState` + `ParaconsistentGate`), 7 tests pass |
| `docs/spec/algebra.md` | the algebraic laws: lattice, absorption, non-coercion theorem |
| `docs/spec/runtime-storage.md` | state layout, gas/memory envelope, monolith-vs-chunking |
| `docs/architecture/subsystems.md` | the kernel→userland OS topology |
| `contracts/` | the 5 satellite contracts (oracle-gateway, ipc-bus, execution-shield, consumer-example, world-registry) |
| `docs/architecture/world-computer-boot.md` | the boot architecture + NEAR cost model |
| `docs/architecture/aivm-substrate.md` | the AI VM substrate: model registry + paraconsistent ALU + execution gate |
| `docs/spec/model-manifest.md` | the Universal Model Manifest (content-addressed weight trees) |
| `docs/spec/inference-invariant.md` | MATCH→TRUE, DIVERGENT→BOTH, TIMEOUT→NEITHER |
| `reference/micro_infer.c` | the on-chain micro-inference engine (BitNet ternary, 668 B wasm, KAT 64/64) |
| `docs/spec/vm-interface.md` | the ZXV VM discovery interface (vm_info / vm_capability / vm_capabilities) |
| `docs/spec/adaptation-protocol.md` | the four-step probe sequence protocols run to auto-adapt |
| `tools/vm_probe.py` | the discovery probe client (verified: gate live) |
| `docs/architecture/adoption-flywheel.md` | the compatibility matrix + the zero-marketing adoption mechanism |
| `tools/standards_check.py` | verifies all 10 standard surfaces live — INDEXERS FIND US |
| `tools/world_boot.py`, `tools/world_install.py` | build/publish the boot manifest; the installer flow |

## The live proof

- **Contract:** `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
- **Code hash (current):** `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB` — chain: `6N7d4cDC…` (v15) → `AiND1AEh…` (audit) → `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB` (ft_transfer_call fix, TX `J21zCZ13…`, state preserved)
- **On-chain:** `lpres_join(kjoin, TRUE, FALSE)` → `3` (`BOTH`)

Security property (verified on-chain, all 16 cells): **no combination
of inputs can coerce a `BOTH` (dispute) or `NEITHER` (unobserved) state
into `TRUE` (authorization).** Only `TRUE` settles.

## The four-valued model

| Code | State | Meaning |
|---|---|---|
| `0x00` | `NEITHER` | Unobserved / Null |
| `0x01` | `TRUE` | Verified assertion |
| `0x02` | `FALSE` | Refuted assertion |
| `0x03` | `BOTH` | Contradictory / disputed evidence |

`kjoin(TRUE, FALSE) = BOTH` — a contradiction is **recorded, never
resolved by picking a side**; and `BOTH` never authorizes.

## The reference implementation

`reference/` carries the exact gate the NEP specifies, extracted from
the ZXV kernel: `lpres_kjoin = (a|b)&3`, conjoin/disjoin by chain rank,
and `lpres_is_settleable` (only TRUE authorizes). The test verifies
**66 checks against the live on-chain proof** — the kjoin table matches
mainnet cell-for-cell, and no input coerces BOTH/NEITHER into TRUE.

```sh
cd reference && ./build.sh     # host test PASS + wasm32 artifact
```

## License

Apache-2.0. Copyright (c) 2024-2026 Michael Laurence Curzi and
36N9 Genetics, LLC.