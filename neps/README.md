# The NEP Submission Package

**"Standard Interface for Deterministic Panic-Free State Resolution
(Chain-Meet)" — everything a reviewer needs.**

This folder accompanies the NEP submission to `near/NEPs`. It is
organized so that an automated reviewer, an SDK maintainer, or a
working-group member can verify every claim without hunting for files.

---

## What's here

| File | Purpose |
|---|---|
| **`nep-paraconsistent-community-utility.md`** | The NEP proposal itself (the document to submit as `neps/nep-0000-panic-free-resolution.md`) |
| **`reference-implementation/nep_gate.rs`** | The minimal, self-contained reference implementation of the standard interface — `#![no_std]`, zero allocations, 8/8 tests passing |
| **`verification.md`** | The reproducible verification guide: on-chain curl commands + the local test suite + the C demonstration + the fuzzing record |

## The quick start for reviewers

```bash
# 1. Verify the live mainnet claim (gas-free)
curl -s https://rpc.mainnet.fastnear.com -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf","method_name":"is_settleable","args_base64":"eyJzdGF0ZSI6IkJPVEgifQ=="}}'
# → false

# 2. Verify the reference implementation (8/8 tests)
cd reference-implementation && cargo test --release
```

## The addresses

- **Canonical (routing):** `zedec.near`
- **The protocol (110 kits):**
  `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`
- **The registry (136 capabilities):**
  `7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`
- **The gate (kernel invariant):**
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`

## The companion repositories

- **The public-utility repository:** https://github.com/sykomiked/zxv-community-utility
  (the user manual, the demonstration guide, the 30-language documentation)
- **The stack:** https://github.com/sykomiked/zxv-stack
- **The developer kit:** https://github.com/sykomiked/zxv-developer-kit
- **The NEP + ledger:** https://github.com/sykomiked/ZXV-NEP-paraconsistent

---

*The system is open. Verify it yourself.*