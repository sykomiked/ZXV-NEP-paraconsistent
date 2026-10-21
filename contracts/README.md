# ZXV subsystem contracts

The userland OS on top of the paraconsistent kernel — four near-sdk-rs
contracts. Compile with the standard NEAR toolchain (`cargo near build`,
near-sdk 5). The embedded `LogicState` + kjoin semantics are verified
identical to the kernel's gate (`reference/lpres.rs`, 7 tests) and to
the on-chain proof (48 cells, 0 coerced).

| Contract | Role | Key invariant |
|---|---|---|
| `oracle-gateway` | dispute arbiter / ingestion | disputes → BOTH, never revert |
| `ipc-bus` | multi-agent messaging | only TRUE tasks become Ready |
| `execution-shield` | settlement proxy (vault) | only TRUE transfers, else halted |
| `consumer-example` | cross-contract validation | promise to kernel `lpres_join` |

See `docs/architecture/subsystems.md` for the full stack.