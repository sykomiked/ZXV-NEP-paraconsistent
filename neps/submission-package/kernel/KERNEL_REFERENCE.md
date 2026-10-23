# The Kernel's NEAR Architecture — Reference Note

**Honest QC finding:** the kernel's NEAR launch file (`near_contract.c`)
is part of a **multi-file kernel build** — it includes 7 headers
(`near_env.h`, `triple_ledger.h`, `vino.h`, `scp144.h`, `sha256.h`,
`lpres.h`, `tribute.h`). The earlier package draft shipped only 3 of
those files, which made the excerpt **not standalone-compilable**. That
was a packaging defect, not a kernel defect.

**The fix:** the kernel sources remain in their repository (the
canonical source of truth), and this submission references them rather
than shipping a partial build.

## Where the complete kernel lives

- **The kernel (full source, including the NEAR architecture):**
  `github.com/sykomiked/zxv-stack` — the `kernel/arch/near/` directory
  contains `near_contract.c`, `near_rt.c`, `near_env.h`, and the full
  kernel include tree (`triple_ledger.h`, `vino.h`, `scp144.h`,
  `sha256.h`, `lpres.h`, `tribute.h`).
- **The deployed contract (the compiled artifact):** the kernel's
  NEAR port is deployed at the gate
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
  and the protocol
  `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`.
  Its code hash is verifiable on-chain.

## What this NEP actually depends on

The NEP's reference implementation (`reference-implementation/
nep_gate.rs`) and the `zxv-nep-gate` crate are **self-contained** — they
compile and test standalone (9/9 tests). The kernel C source is the
**provenance** of the logic, not a build dependency of the standard.

## The verification path for reviewers

1. The **standard interface** is verified by the self-contained crate
   (`cargo test --release` → 9/9).
2. The **live deployment** is verified by the on-chain curl commands in
   `02-VERIFICATION-GUIDE.md`.
3. The **kernel provenance** is verified by the repository
   (`github.com/sykomiked/zxv-stack`, the `kernel/arch/near/` source).

*The logic is verified three ways; the kernel source is referenced, not
bundled out of context.*