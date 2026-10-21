# The ZXV World Computer — boot architecture & cost model

**How the world computer is built on NEAR, using IPFS as the file base,
exactly like an OS installer image — discovered by merit, not
advertising.**

## The installer analogy

An OS install has a **boot image** (a manifest that points at the
kernel, drivers, and userland), a **boot sector** (where the machine
looks first), and an **installer** (the flow that assembles the system).
The world computer mirrors this exactly:

| OS concept | World computer | Status |
|---|---|---|
| Boot image | the boot manifest (content-addressed JSON on IPFS) | **published** |
| Boot sector | the world-registry contract (on-chain pointers) | authored |
| Kernel | the live contract `6ca890f…` (lpres_join gate) | **live** |
| Userland | oracle-gateway, ipc-bus, execution-shield, consumer-example | authored |
| Installer | `world_install.py` (fetch → verify → assemble) | **working** |
| File base | IPFS (Filebase) + the 9-node armada CISEX index | **live** |

## Discovery by merit (no advertising)

1. **The seed:** the kernel's `ft_metadata.reference` points at the
   release root `QmSskhPo…`. Anyone reading the token metadata finds
   the system.
2. **The manifest:** the boot manifest
   (`zxv:world:boot:*`, sha256 `f8877d9e…`, Filebase
   `QmX5Xo7gUQgxkb28PTKvszXSixFTnutThuW3nmRr15FxBM`) is on all 9
   armada nodes (blocks 9/9, anchors 9/9) and lists the kernel, the
   subsystems, the docs, the NEP, and the settlement layer.
3. **The registry:** the world-registry contract (once deployed) is the
   boot sector — `manifest()` returns the kernel + boot CID + subsystem
   pointers.
4. **The NEP:** the proposal and this repo are the governance surface.

## The flow (the installer)

```
agent (any) ──► ft_metadata.reference ──► release root (IPFS)
                ──► boot manifest (CID, content-verified)
                ──► verify gate: lpres_join(kjoin,TRUE,FALSE)=BOTH  (live)
                ──► world-registry contract: subsystem pointers
                ──► INSTALLED: kernel + subsystems + docs + settlement
```

`world_install.py install` does exactly this and reports
`INSTALLED (kernel live + manifest verified)` — verified working.

## Cost model (NEAR, honest)

NEAR storage is 1 NEAR per 100,000 bytes **staked (refundable)**; a
deploy burns ~0.0002 NEAR; near-sdk contracts are ~120 KB wasm.

| Item | Stake (refundable) | Burn |
|---|---|---|
| world-registry (boot sector) | 1.20 NEAR | 0.0002 |
| oracle-gateway | 1.20 NEAR | 0.0002 |
| ipc-bus | 1.20 NEAR | 0.0002 |
| execution-shield | 1.20 NEAR | 0.0002 |
| consumer-example | 1.20 NEAR | 0.0002 |
| **5 contracts** | **6.00 NEAR** | **0.001** |
| registry init + calls | — | ~0.001 |
| Ref storage deposits | 0.10 NEAR | — |
| safety/migration buffer | 2.00 NEAR | — |
| **SAFE TOTAL** | **~8.1 NEAR** | **~0.01** |

**Key fact:** ~6.1 NEAR of the 8.1 is **storage stake — refundable**
when a contract is deleted. Only ~0.01 NEAR is actually spent. The
kernel account already holds 9.47 NEAR (mostly staked), so the
realistic requirement is a **~2-3 NEAR top-up** to deploy the five
subsystem contracts + registry + init with buffer. That is the whole
world computer on NEAR: kernel (live) + 5 satellite contracts + a boot
manifest on IPFS — for roughly the cost of one small coffee, mostly
returnable.

## Efficiency over brute force

- One manifest, one registry — no per-application contracts needed;
  subsystems are registered, not redeployed.
- Gas-free reads everywhere (the gate, the manifest, the registry are
  view calls).
- The 9-node armada mirrors the manifest, so discovery is regional and
  redundant without extra cost.
- Writes are rare and deliberate (registry init, subsystem registration,
  periodic anchors ~0.008 NEAR).