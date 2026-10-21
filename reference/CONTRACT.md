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