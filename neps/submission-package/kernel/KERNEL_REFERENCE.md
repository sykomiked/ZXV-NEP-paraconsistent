# The Foundational Paraconsistent Kernel — included in this package

**The kernel's paraconsistent logic module (`lpres`) — the exact logic
this NEP standardizes — is included here, complete and buildable.**

## What's included

```
kernel/
├── lpres/                  ← the foundational paraconsistent kernel module
│   ├── lpres.c             ← the four-valued logic operations (conjoin/disjoin)
│   ├── lpres.h             ← the public interface (lpres_state_t)
│   ├── lpres_core.c/.h     ← the core state machine
│   ├── lpres_adapter.c/.h  ← the adapter layer
│   ├── lpres_bridge.c/.h   ← the bridge layer
│   ├── lpres_hex.c/.h      ← the hex/encoding helpers
│   ├── lpres_phases.h      ← THE RANK TABLE (FALSE=0 < NEITHER=1 < BOTH=2 < TRUE=3)
│   └── test_lpres*.c       ← the module tests
├── near_contract.c         ← the NEAR smart-contract entry (the deployed kernel port)
├── near_env.h              ← the NEAR host environment bindings
└── near_rt.c               ← the NEAR runtime helpers
```

## The build recipe (verified)

The module compiles clean with the kernel's canonical include path:

```bash
gcc -c -std=c11 \
  -I src/lpres -I src/finance -I src/vino -I src/scp144 \
  -I src/robin_debanks -I src/tribute -I src/event_space \
  -I src/e8 -I src/modbind -I include -I src/surplus \
  src/lpres/lpres.c        # 0 errors
src/lpres/lpres_adapter.c  # 0 errors
src/lpres/lpres_bridge.c   # 0 errors
src/lpres/lpres_core.c     # 0 errors (needs src/surplus in CPATH)
src/lpres/lpres_hex.c      # 0 errors
```

## The authoritative rank table (lpres_phases.h)

```c
static const uint8_t lpres_rank[LPRES_STATE_COUNT] = {
    [LPRES_STATE_FALSE]   = 0,  /* the bottom */
    [LPRES_STATE_NEITHER] = 1,
    [LPRES_STATE_BOTH]    = 2,
    [LPRES_STATE_TRUE]    = 3,  /* the top — only TRUE settles */
    [LPRES_STATE_GATEWAY] = 4,
    /* ... 145 states total ... */
};
```

This is the exact chain-meet the NEP standardizes: the meet returns
the smaller rank, so no non-TRUE combination can ever yield TRUE.

## What is NOT bundled (honest note)

The NEAR contract (`near_contract.c`) also includes `triple_ledger.h`,
`vino.h`, `scp144.h`, `sha256.h`, and `tribute.h` — the supporting
kernel modules (finance, VM, lattice, hashing, tribute). Those live in
the kernel repository (`github.com/sykomiked/zxv-stack`) and are the
deployment's supporting infrastructure, not the paraconsistent core.
The **foundational paraconsistent logic** — the subject of this NEP —
is fully included and buildable above.

## The deployed artifact

The kernel port is deployed at the gate
`6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
and the protocol
`f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`;
its code hash is verifiable on-chain.

*The system is open. Verify it yourself.*