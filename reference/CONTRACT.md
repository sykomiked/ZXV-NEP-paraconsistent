# The live contract — exact surface the NEP specifies

The deployed reference implementation runs on NEAR mainnet:

| Field | Value |
|---|---|
| Contract account | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` |
| Code hash (verified 2026-10-20) | `6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU` |
| Build | `build_system/build_near_contract.sh` in the ZXV source repo |
| Compiler | clang 23.1.0 `--target=wasm32-unknown-unknown`, freestanding, zero-libc |
| Explorer | https://nearblocks.io/address/6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8 |

## The gate method (exported, gas-free view)

```
lpres_join(op, a, b) -> u8
  op: 0 = conjoin (meet), 1 = disjoin (join), 2 = kjoin (knowledge join)
  a, b: 0 NEITHER, 1 TRUE, 2 FALSE, 3 BOTH
```

On-chain, callable by anyone:

```bash
# kjoin(TRUE, FALSE) = BOTH — the contradiction is retained
curl -s https://rpc.mainnet.near.org -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"z","method":"query","params":{
       "request_type":"call_function","finality":"final",
       "account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8",
       "method_name":"lpres_join","args_base64":"AgEC"}}'
# -> result byte 3 (BOTH)
```

## Reproduce the reference artifact (transparency)

`reference/build.sh` compiles `lpres_gate.c` to wasm32 with the same
flags as the deployed contract and runs the 66-check host test. The
resulting `lpres_gate.wasm` hash is:

- **SHA-256 (built 2026-10-20):** `20267421aa233f2b776dcc5f881363561c54b622e88104fdf6ea3afc2da06067`

Rebuild it (`cd reference && ./build.sh`) and compare — the gate is
deterministic, so the hash should match. The full kernel binaries are
NOT committed to this repo (a reviewer cannot review a blob); the
reviewable artifacts are the source, the test, and this build path.

## The ZXV VM — live mainnet deployment (2026-10-20)

The complete system is deployed to NEAR mainnet. Every contract runs on
its own implicit account (keypairs verified before funding).

| Layer | Contract | Mainnet account | Status |
|---|---|---|---|
| Root kernel (audited) | the paraconsistent gate + ledger | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` | LIVE |
| Boot sector | world-registry (vm_info/vm_capability) | `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a` | LIVE |
| Dispute arbiter | oracle-gateway | `6a57f581ea7c7c986769af6c874b16e8187410b62fe7190cdfb06001fb8998aa` | LIVE |
| Messaging | ipc-bus | `3c70e9c629068ac1fad6bd61ea26d4b4486c7e2a033c87d3635051bc2db0276e` | LIVE |
| Settlement vault | execution-shield | `789457e61798d8deb3e45d60e84dec1ba0ca0fa8e7181eaa1646485f0ea89846` | LIVE |
| AIVM micro-inference | micro-infer | `86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe` | LIVE |
| AIVM model ingestion | model-registry | `77361b15a4a8f7e43fdbf7b276138eeaf9d8fb5f6c2e73ec25c8c8b76a719c65` | account created, deploy pending |

- **Kernel code hash (audited):** `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB`
- **Discovery:** `vm_info()` on the world-registry returns the kernel,
  8 capabilities, and the boot manifest CID `f8877d9e…`.
- **On-chain proof:** `lpres_join(kjoin,TRUE,FALSE)=BOTH` on mainnet.

## The kernel source (in the ZXV repository)

- `kernel/src/lpres/lpres.c` — `lpres_kjoin`, `lpres_conjoin`,
  `lpres_disjoin`, `lpres_implies_actuator_authority` (the gate)
- `kernel/src/lpres/lpres_phases.h` — the state enum
  (`NEITHER 0 / TRUE 1 / FALSE 2 / BOTH 3`)
- `kernel/arch/near/near_contract.c` — the exported `lpres_join`
  view method (line ~672)
- Source repo: https://github.com/sykomiked/ZEDEC-XERO-VULPINE-V1.0.0

This repo (`ZXV-NEP-paraconsistent`) carries a standalone copy of the
gate in `reference/lpres_gate.c` so the NEP is self-contained.