# Demonstration: Deterministic AI on NEAR — the ZXV Micro-Inference Substrate

**Category: Show and Tell — a live, verifiable mainnet subsystem.**

This is a running cluster of 8 contracts on NEAR mainnet. It
demonstrates integer-exact, deterministic AI forward evaluation and
four-valued paraconsistent dispute resolution — all callable today,
all gas-free reads, no wallet, no funding.

---

## 1. What is running (all deployed, all live, 2026-10-20)

| Contract | Account | Role |
|---|---|---|
| **micro-infer** | `86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe` | deterministic on-chain inference engine |
| **kernel** (the gate) | `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` | four-valued paraconsistent logic gate (`lpres_join`) + the root state ledger |
| **world-registry** | `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a` | `vm_info()` — the discovery surface (8 capabilities) |
| **oracle** | `6a57f581ea7c7c986769af6c874b16e8187410b62fe7190cdfb06001fb8998aa` | paraconsistent dispute arbiter (TRUE+FALSE→BOTH, never reverts) |
| **shield** | `789457e61798d8deb3e45d60e84dec1ba0ca0fa8e7181eaa1646485f0ea89846` | settlement vault — only TRUE authorizes a payout |
| **bus** | `3c70e9c629068ac1fad6bd61ea26d4b4486c7e2a033c87d3635051bc2db0276e` | multi-agent IPC (publish/attest/consume) |
| **model-registry** | `77361b15a4a8f7e43fdbf7b276138eeaf9d8fb5f6c2e73ec25c8c8b76a719c65` | content-addressed model loading (manifest → weight tree) |
| **consumer** | `8947dcb7d35102dca4dc6cb852038b17efa1ff1a421209e5d0e7122425b0c654` | the cross-contract proof — a standard near-sdk-rs contract |

The contracts are separated by concern: the kernel owns the gate and
ledger; the satellites own arbitration, gating, messaging, model
loading, and the integration proof. This is not one contract doing
custom memory math — it is an integrated subsystem.

## 2. Verify it yourself (zero-gas view calls)

All of the following were executed against mainnet and returned the
shown values. Every read is a gas-free view call.

### 2a. Deterministic inference (micro-infer)

```bash
near view 86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe infer \
  '{"token": 3, "dim": 4}'
# -> 3
```

Run it again: same result. From any machine: same result. That is the
property that makes on-chain AI different — deterministic and
integer-exact, so outputs never drift across hardware.

### 2b. The known-answer test (64/64)

```bash
near view 86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe check \
  '{"token": 5, "dim": 8}'
# -> 1
```

All 64 cells of the KAT pass on-chain (verified 2026-10-20). The
expected values are computed from the model table itself — see
`reference/micro_infer.c` in the repository for the ground truth.

### 2c. The paraconsistent gate — the contradiction (raw bytes)

`lpres_join` on the kernel reads **raw binary args** (op, left, right),
so the copy-paste proof is a `curl` (the base64 `AgEC` = bytes
`[2, 1, 2]` = op 2/kjoin, left TRUE, right FALSE):

```bash
curl -s https://rpc.mainnet.fastnear.com -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{
        "request_type":"call_function","finality":"final",
        "account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8",
        "method_name":"lpres_join","args_base64":"AgEC"}}'
# -> the result byte is 0x03 = BOTH
```

`TRUE + FALSE = BOTH`: the contradiction is recorded, never coerced,
never a deadlock. Settlement requires exactly `TRUE`.

### 2d. The discovery surface (the whole OS in one call)

```bash
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_info '{}'
# -> ZXV Virtual Machine, 8 capabilities, each resolving to a real account
```

### 2e. The cross-contract proof (consumer → kernel)

The consumer is a standard near-sdk-rs contract. It fired a
cross-contract promise to the kernel's `lpres_join`; the kernel's
verdict returned as `Aw==` (base64 of byte 0x03 = BOTH):

```bash
near view 8947dcb7d35102dca4dc6cb852038b17efa1ff1a421209e5d0e7122425b0c654 kjoin \
  '{"a": "True", "b": "False"}'
# -> "Both"
```

This settles the integration question: any near-sdk-rs contract can
call the gate across a promise and receive the paraconsistent verdict
without a panic.

## 3. What the micro-inference model actually is

`micro-infer` runs a **BitNet-1.58-style ternary lookup embedding**:
an 8×8 table of ternary values (−1, 0, +1), packed 2 bits per cell
(64 bytes on-chain — `EMBED[64]` in `reference/micro_infer.c`).
`infer(token, dim)` is the integer dot of the token's ternary row over
`dim` columns — pure integer arithmetic, no floating point, no
division, no non-determinism.

**Scope boundary (important):** this is the **on-chain micro-inference
path** — models small enough to embed (sub-5 MB) run entirely inside
the contract, deterministically. Large multi-billion-parameter models
are a separate path: their **manifests** (weight-tree Merkle roots,
quantization, URIs) are registered on-chain in `model-registry`, and
the heavy forward pass runs off-chain on the compute mesh, with layer
commitments submitted back for verification. The NEP repo documents
both (`docs/spec/model-manifest.md`,
`docs/spec/inference-invariant.md`). This demo is the provable half.

## 4. Why this matters

Every AI today is a black box: call an API, trust the answer. The ZXV
micro-inference engine is the opposite — the model, the execution, and
the result are all on-chain, integer-exact, and verifiable by anyone.
Same inputs → same output → same receipt, on any machine, forever.
That is the foundation of an AIVM: a world computer whose AI outputs
are provable, not trusted. And when two AI sources disagree, the
four-valued gate records the dispute as BOTH instead of deadlocking —
the settlement invariant (only TRUE moves value) keeps the machine
safe.

## 5. Source and reproducibility

- Repository: `https://github.com/sykomiked/ZXV-NEP-paraconsistent`
  - `reference/micro_infer.c` — the C reference (KAT ground truth)
  - `contracts/micro-infer/` — the deployed near-sdk-rs contract
  - `docs/spec/algebra.md` — the four-valued lattice and non-coercion proof
  - `docs/evidence/lpres_join_onchain_proof.json` — the 48-cell on-chain proof
- Build: `cargo near build` (near-sdk 5.9, rustc 1.86) — the deployed
  wasm is byte-verifiable against the source.

---

*All values above were verified against NEAR mainnet on 2026-10-20.
Nothing in this demonstration requires funding, a wallet, or a
deposit — every read is a gas-free view call.*