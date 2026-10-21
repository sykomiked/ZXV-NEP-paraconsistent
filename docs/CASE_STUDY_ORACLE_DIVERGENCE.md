# Oracle Divergence: Before vs. After — the paraconsistent settlement gate

**The one-page case study. The question every DeFi/AI team asks: "what
happens when two sources disagree?"**

## BEFORE: classical logic (every standard contract today)

Two oracle feeds disagree on a price. The contract sees two different
values where it expected one. Classical logic has exactly one answer:
**panic.**

| Step | What happens | Cost |
|---|---|---|
| 1 | `env::panic_str("oracle divergence")` | the transaction **reverts** |
| 2 | All state changes roll back | **gas is burned** (not refunded) |
| 3 | The liquidation/payout never fires | users stuck, positions stale |
| 4 | Downstream calls fail too | cascading deadlock |

The failure is **binary and expensive**: either the whole thing works,
or the whole thing burns gas and dies. A single bad feed can deadlock a
lending protocol, freeze a bridge, or halt an agent's trade loop.

**The exploit vector:** a manipulated or buggy oracle diverges → the
protocol reverts or liquidates innocent users at the wrong price. This
is the $10M-class bug class that oracle incidents keep producing.

## AFTER: the paraconsistent gate (ZXV)

The same divergence, through the four-valued gate:

| Step | What happens | Cost |
|---|---|---|
| 1 | `kjoin(TRUE, FALSE) = BOTH` (0x03) | **no panic** — the state is recorded |
| 2 | The contradiction is **held in register** | near-zero gas, no rollback |
| 3 | Settlement is refused (`BOTH` is not settleable) | **the liquidation/payout is frozen**, not deadlocked |
| 4 | Normal trading continues on other pairs | the protocol stays alive |
| 5 | Evidence resolves it → `TRUE` → settle | the receipt is the proof |

**The key difference:** a divergence no longer *kills* the contract —
it *pauses one decision*. Everything else keeps running. Gas is not
burned on rollback; the state is held, not destroyed.

## The one-line integration

```rust
// before: env::panic_str("oracle divergence");  // burns gas, reverts
// after:
use near_paraconsistent::{Gate, LogicState, settle_or_hold};

let verdict = settle_or_hold(&[feed_a, feed_b])?; // Err(BOTH) = held, no panic
```

or, from an agent:

```ts
const verdict = await shield.evaluate([modelA, modelB]);
if (!verdict.canSettle()) {
  // divergence contained: state held as BOTH (0x03) — no crash, no rollback
}
```

## The numbers that matter

| Metric | Classical | Paraconsistent |
|---|---|---|
| Divergence result | revert (panic) | held as `BOTH` |
| Gas on conflict | burned (rollback) | near-zero (register hold) |
| Settlement on `BOTH` | N/A | **refused by construction** |
| Majority can coerce? | N/A | **no** — BOTH is absorbing |
| Rest of protocol | deadlocked | keeps running |

## The pitch

> "Show a lending protocol that this stops a $10M exploit vector with
> one line of configuration — adoption becomes obvious."

**One line.** No new infrastructure on their side — the gate is live on
NEAR mainnet, discoverable via `vm_capabilities()`, callable by any
contract. Divergence becomes a pause, not a panic. Contradiction
becomes information, not failure.

---

*The gate: `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8` (lpres_join) · the discovery registry: `7eaa2c6f…` (vm_capabilities) · the Rust crate: `near-paraconsistent` · the TS SDK: `@zxv/agent-guard`.*