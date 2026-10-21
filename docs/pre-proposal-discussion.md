# Pre-Proposal Discussion — Paraconsistent Logic Resolution for Multi-Agent On-Chain State Machines

**Category:** Standards

## The problem

Multi-agent and intent-driven architectures (NEAR AI agents, oracle-fed
contracts, autonomous workflows) routinely receive **contradictory
inputs** — two oracles disagree, a model flips, a source refutes
another. Classical Boolean logic forces a choice: revert (lose the
transaction) or deadlock (lose the workflow). Neither is acceptable for
an autonomous economy.

## The proposal

A **four-valued paraconsistent logic gate** (`lpres_join`) that lets a
contract **record contradictions instead of exploding**:

- `NEITHER (0)` — unobserved
- `TRUE (1)` — verified
- `FALSE (2)` — refuted
- `BOTH (3)` — disputed (TRUE + FALSE = BOTH, retained)

The settlement invariant: **only `TRUE` authorizes a financial action.**
A `BOTH` state is recorded and isolated — never coerced into
authorization, never silently resolved by picking a side.

## Live proof-of-concept on NEAR mainnet

- **Contract:** `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
- **Code hash (verified 2026-10-20):** `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB`
- **On-chain, gas-free, callable by anyone:**
  `lpres_join(kjoin, TRUE, FALSE)` → `3` (`BOTH`)

```bash
curl -s https://rpc.mainnet.near.org -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"z","method":"query","params":{
       "request_type":"call_function","finality":"final",
       "account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8",
       "method_name":"lpres_join","args_base64":"AgEC"}}'
# returns the byte 3 (BOTH)
```

The implementation is a freestanding C kernel compiled to Wasm32
(zero-libc), running the identical paraconsistent logic the ZXV kernel
uses across its multi-node mesh — integer-exact, deterministic,
Apache-2.0.

## What I'm asking the community

1. Is the four-valued gate a useful **contract standard** for NEAR AI
   and oracle-fed contracts?
2. Should `lpres_is_settleable` (the TRUE-only settlement gate) be part
   of the same interface, or separate?
3. Does the working group want a reference Rust (`near-sdk-rs`) binding
   alongside the C/Wasm reference implementation?

## Links

- Forensic ledger (IPFS): `QmSFZ7W7VGoadJLhd8ARDRpukA8XCKaQ9YtjnxLgbEUSQX`
- System documentation (IPFS): `QmPVXB22kX1HSSXRn76GfBgBLwFqfMtPNbgR9mHJkD7PiU`
- Source: https://github.com/sykomiked/ZEDEC-XERO-VULPINE-V1.0.0
- Proposal repo: https://github.com/sykomiked/ZXV-NEP-paraconsistent
---

## LIVE ON MAINNET — the proof is on-chain (2026-10-20)

The full system is deployed and testable right now:

- **The gate:** `lpres_join(kjoin, TRUE, FALSE) = BOTH` — verified live on
  mainnet (contract `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`,
  code hash `AiND1AEhrQVhEBpAdpkpRUcLjthf18qswYwobideWSKB`; previous
  audited hash `AiND1AEh…`, upgraded by TX `J21zCZ13…` with state
  preserved — the old hash leads to this one).
- **Cross-contract proof:** a standard near-sdk-rs contract
  (`consumer`) fires a promise to the kernel's `lpres_join` and receives
  BOTH back — the integration pattern any contract can use.
- **The discovery surface:** `vm_info()` on the world-registry
  (`7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`)
  returns the whole OS: 8 capabilities, each resolving to a real
  deployed account. Any protocol adapts by probing — no integration
  contract, no marketing needed.
- **The subsystems (all live):** oracle (dispute arbiter), bus (IPC),
  shield (TRUE-only settlement vault), micro-infer (on-chain AI wasm),
  model-registry (content-addressed model loading), consumer
  (cross-contract validation).
- **Xahau settlement:** the treasury's accounting hook is deployed at
  chain position 0 and fires on payments; the bridge tracks settlement
  flows to IPFS/NEAR.

**Try it yourself (free, gas-free views):**
```
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_info '{}'
```
