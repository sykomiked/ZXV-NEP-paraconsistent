# The NEP Submission Package

**"Standard Interface for Deterministic Panic-Free State Resolution
(Chain-Meet)" — the complete attachment set for the `near/NEPs`
pull request.**

---

## What's in this package

```
submission-package/
├── 01-NEP.md                      ← the NEP proposal (submit as
│                                    neps/nep-0000-panic-free-resolution.md)
├── 02-VERIFICATION-GUIDE.md       ← reproducible verification: on-chain
│                                    curl commands + the local test suite
├── reference-implementation/
│   └── nep_gate.rs                ← the minimal reference implementation
│                                    (#![no_std], zero allocations)
├── crate/                         ← the zxv-nep-gate Rust crate
│   ├── Cargo.toml                 ← publish-ready (crates.io shape)
│   ├── README.md
│   └── src/lib.rs                 ← 9/9 tests passing; builds for wasm32
├── contracts/
│   ├── zxv-core-contract.rs       ← the deployed 110-kit contract source
│   └── zedec-near-canonical.rs    ← the canonical router (zedec.near) source
├── kernel/
│   ├── lpres/                     ← THE FOUNDATIONAL PARACONSISTENT
│   │   │                             KERNEL (complete, buildable):
│   │   │                             lpres.c (conjoin/disjoin),
│   │   │                             lpres_phases.h (the rank table),
│   │   │                             lpres_core/adapter/bridge/hex
│   │   ├── near_contract.c        ← the NEAR launch file (the C kernel
│   │   │                             deployed on NEAR)
│   │   ├── near_rt.c
│   │   └── near_env.h
└── ledger/
    ├── ADDRESS_LEDGER.md          ← every contract address + role
    └── TUTORIAL.md                ← how to use the whole system
```

## The honest note on the contract sources

The deployed contracts' **bytecode is already public on NEAR mainnet** —
anyone can read it, call it, and verify the code hashes. Including
their **source** here is therefore not a disclosure; it is what makes
the NEP's "working proof" fully verifiable: a reviewer can build the
source and compare the deployed code hash. The kernel's NEAR
architecture (`near_contract.c`) is the C kernel that launched on NEAR.

**What is deliberately NOT in this package:** keys, credentials,
deployment secrets, and the deploy environment configuration. Those
remain private.

## The quick start for reviewers

```bash
# 1. Verify the live mainnet claim (gas-free)
curl -s https://rpc.mainnet.fastnear.com -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf","method_name":"is_settleable","args_base64":"eyJzdGF0ZSI6IkJPVEgifQ=="}}'
# → false

# 2. Verify the crate (9/9 tests)
cd crate && cargo test --release

# 3. Verify the no_std claim (builds for wasm32)
cargo build --release --target wasm32-unknown-unknown
```

## The addresses (in the ledger)

- **Canonical (routing):** `zedec.near`
- **The protocol (110 kits):**
  `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`
- **The registry (136 capabilities):**
  `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`
- **The gate (kernel invariant):**
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`

---

*The system is open. Verify it yourself.*