# Distributed OS Topology — kernel + userland subsystems

**The stack that turns the standalone kernel contract into a functional
distributed operating system.** A kernel alone does not make an OS: the
world-computer experience requires userland services and
application-facing APIs on top of the root invariants.

```
┌────────────────────────────────────────────────────────────────────┐
│            Userland / Application Layer                            │
│   (NEAR AI swarms, Shade agents, intent solvers, DAOs)             │
└───────────────────────────────────┬────────────────────────────────┘
                                    │ cross-contract calls (promises)
                                    ▼
┌────────────────────────────────────────────────────────────────────┐
│            Subsystem Contracts (the userland OS)                   │
│                                                                    │
│  [1. Oracle Gateway]  [2. IPC Bus]      [3. Execution Shield]      │
│  dispute arbiter      messaging bus     settlement proxy (vault)   │
│  ingest + lpres_join  publish/subscribe gated transfers (TRUE-only)│
└───────────────────────────────────┬────────────────────────────────┘
                                    │ low-level host queries
                                    ▼
┌────────────────────────────────────────────────────────────────────┐
│            Root Kernel Layer (live on NEAR mainnet)                │
│   account 6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8
│   "zxv:v1" monolithic state · freestanding Wasm32 C kernel         │
│   lpres_join / lpres_is_settleable — the four-valued root gate     │
└────────────────────────────────────────────────────────────────────┘
```

## The three satellite contracts (this repo: `contracts/`)

### 1. `oracle-gateway` — Autonomous Agent Dispute Arbiter

**Role:** the ingestion endpoint. External agents do not read the
kernel's 898 KB state blob; they post assertions here. The gateway
joins evidence (kjoin), isolates disputes as `BOTH` without reverting
the caller, and emits a log event only when a claim resolves cleanly to
`TRUE`.

**Key methods:** `open_claim`, `assert`, `query`, `is_settleable`.

### 2. `ipc-bus` — Multi-Agent IPC Bus

**Role:** the messaging subsystem (POSIX pipes/sockets on-chain).
Agents publish tasks and subscribe to notifications. Paraconsistent
gating prevents conflicting instructions from jamming the queue:
verdict `TRUE` → task `Ready`; `BOTH`/`FALSE` → `Blocked` (quarantined,
never executed, workflow preserved); `NEITHER` → `Queued` (waits).

**Key methods:** `publish`, `attest`, `subscribe`, `status`, `consume`.

### 3. `execution-shield` — Settlement Proxy (vault)

**Role:** the capital-gating subsystem. A NEAR vault whose every
outbound transfer is gated strictly behind
`lpres_is_settleable(state) == TRUE`. An agent attempting a payout in a
disputed/unverified state gets `false` — funds halted, workflow NOT
aborted, attempt counted for audit.

**Key methods:** `deposit`, `attempt_payout`, `balance`, `counters`.

### + `consumer-example` — cross-contract validation

**Role:** the integration proof. A minimal near-sdk-rs contract that
fires a promise calling `lpres_join` on the kernel
(`6ca890f…`) and stores the returned verdict in its callback — the
exact pattern the working group will test.

## Design rules

1. **Lightweight, idiomatic Rust.** All satellites are standard
   `near-sdk-rs` with the `LogicState` enum and the gate trait from
   `reference/lpres.rs`. Mainstream developers integrate with the
   kernel via standard code, not the raw C ABI.
2. **The kernel is the root; the subsystems enforce.** The gate's
   invariants live in the kernel (root of truth); the shield and bus
   enforce them at the points capital and execution actually flow.
3. **Never revert on contradiction.** Disputes resolve to `BOTH` and
   are isolated — the caller's workflow is preserved, the funds are
   held, the task is quarantined.
4. **Audit by construction.** Payouts, blocked attempts, and task
   transitions are counted and logged.

## Status

- `reference/lpres.rs` — verified (7 tests, `cargo test`).
- `contracts/*` — written in idiomatic near-sdk-rs; compile with the
  standard NEAR toolchain (`cargo near build`), the near-sdk 5
  contract workflow. The gate logic they embed is identical to the
  verified reference.
- Kernel live on mainnet: `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
  (code hash `315cfVxvveFBrT6FnT4Cr27AoQrNSyYG1XF8Lu13ZCCF`).