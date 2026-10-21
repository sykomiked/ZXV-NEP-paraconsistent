# ZXV — The Paraconsistent World Computer: Mainnet Deployment Manifest

**Status update — 2026-10-21**

The ZXV stack has moved from proposal to **live mainnet infrastructure**. The four-value paraconsistent logic gate (`TRUE + FALSE = BOTH`, only `TRUE` settles) is deployed, registered, and discoverable on NEAR mainnet — 13 subsystems, each resolving to a live contract.

## The white paper

The full white paper is published and content-addressed:

- **Markdown:** https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/WHITEPAPER.md
- **Canonical IPFS CID:** `QmPiju3iX37GnWkUCVW72KysSSSFY3teAJHaMDgTEumCxQ` (resolves on the public gateway)
- **Forensic ledger (the full audit trail):** https://github.com/sykomiked/ZXV-NEP-paraconsistent/blob/main/docs/ledger/FORENSIC_LEDGER.md

## The live stack — 13 registered subsystems

`vm_capabilities()` on the world-registry (`7eaa2c6f…`) enumerates all 13:

| # | Capability | Account (full 64-hex) |
|---|---|---|
| 1 | gate | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` |
| 2 | ledger | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` |
| 3 | registry | `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a` |
| 4 | oracle | `6a57f581ea7c7c986769af6c874b16e8187410b62fe7190cdfb06001fb8998aa` |
| 5 | bus | `3c70e9c629068ac1fad6bd61ea26d4b4486c7e2a033c87d3635051bc2db0276e` |
| 6 | shield | `789457e61798d8deb3e45d60e84dec1ba0ca0fa8e7181eaa1646485f0ea89846` |
| 7 | micro_infer | `86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe` |
| 8 | model_registry | `77361b15a4a8f7e43fdbf7b276138eeaf9d8fb5f6c2e73ec25c8c8b76a719c65` |
| 9 | consumer | `8947dcb7d35102dca4dc6cb852038b17efa1ff1a421209e5d0e7122425b0c654` |
| 10 | **paraconsistent_oracle** | `ea85f5381ceff17553fa3093a2d67a0e6cc4853abd7c19487ed7a33391acc2f7` |
| 11 | **settlement_router** | `8bf177019402d3d843b361a04d3ed7e21de637bf5d252956a15208f7950effae` |
| 12 | **yhv_wrapper** | `1a051ac2465c9a04e532fe420e2efbd8cc12f9a1d9f2a350470782fd3255ecf1` |
| 13 | **ai_verdict** | `781fc03d4dfb9b29d88df561157b11db6dc1e02368eae6deef7998d188ff966e` |

## The new DeFi layer (testnet-proven → mainnet live)

**1. Paraconsistent Oracle** — the dispute-tolerant price harmonizer. Connects to existing oracle feeds; agreement → `TRUE` (settleable price), divergence → `BOTH` (held, never coerced). Any protocol reads `price(asset)`.

**2. Settlement Router** — the receipt rail. `submit_evidence` → kjoin accumulates → `settle()` only on `TRUE`, emitting a content-addressed receipt. A majority **cannot** coerce a recorded contradiction (verified on-chain).

**3. YHVH Wrapper** — the liquidity adapter. Receives `$YHVH` via the kernel's working `ft_transfer`, mints `wYHVH` 1:1 with a standard-ABI `ft_transfer_call`. This is the scenic bypass: the promoted kernel hash (`AiND1AEh…`) stays pinned while the DEX deposit path is fixed in transit.

**4. AI Verdict** — settleable on-chain AI. Model outputs from multiple sources run through the gate: agreement → `TRUE` (settleable), divergence → `BOTH` (held). Deterministic integer-only micro-inference (64/64 KAT) stays on-chain; heavy weights stay off-chain as content-addressed manifests.

## Verify it yourself (gas-free views)

```bash
# the whole OS in one call
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities '{}'

# the paraconsistent gate: TRUE + FALSE = BOTH
curl -s https://rpc.mainnet.fastnear.com -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8","method_name":"lpres_join","args_base64":"AgEC"}}'

# the settlement invariant
near view 8bf177019402d3d843b361a04d3ed7e21de637bf5d252956a15208f7950effae is_settleable '{"state":"BOTH"}'

# deterministic on-chain AI
near view 86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe infer '{"token":3,"dim":4}'
```

## What this means

The system is no longer a proposal — it is a **deployment manifest**: every contract audited on testnet first, then deployed, funded, and registered. The white paper, the ledger, and the telemetry feeds all carry the current state. The settlement invariant — **only `TRUE` settles; contradictions are recorded, never coerced** — is now live infrastructure any protocol can call.

---

**The repository:** https://github.com/sykomiked/ZXV-NEP-paraconsistent