# Paraconsistent Logic: What It Solves, and a Live Implementation on NEAR

*@developer.near · @devhub.near — a technical post for the developer community*

---

## The problem most systems refuse to face

Every distributed system eventually hits the same wall: **two sources of
truth disagree, and the system must decide what to do.**

The standard answers are all failure modes in disguise:

- **The freeze** — the transaction stalls, the ledger locks, the
  multisig hangs. Disagreement becomes a deadlock.
- **The last-writer-wins** — the newest claim silently overwrites the
  older one. Disagreement becomes data loss.
- **The majority coercion** — the biggest vote force-resolves the
  dispute. Disagreement becomes power, not truth.
- **The panic** — the contract throws, the state reverts, the users
  retry. Disagreement becomes an exception.

All four treat disagreement as a *bug*. None of them preserve the
information that a disagreement contains.

## The paraconsistent answer: disagreement is data

Paraconsistent logic changes the frame: **a contradiction is not an
error state — it is an information state.** The system holds both claims
simultaneously, records the conflict, and refuses to coerce a
resolution until the evidence genuinely settles.

This is the four-valued logic (Belnap-style, adapted):

| State | Meaning | Behavior |
|---|---|---|
| `TRUE` (0x01) | agreed, attested | **settles** |
| `FALSE` (0x02) | explicitly refuted | refused |
| `NEITHER` (0x00) | no evidence yet | waits |
| `BOTH` (0x03) | contradiction, held | **recorded, never coerced** |

The invariants that fall out:

- **Only TRUE settles.** Nothing moves on a contradiction.
- **BOTH is absorbing.** A majority can never coerce a recorded
  contradiction — no vote can force `BOTH` to become `TRUE`.
- **Disagreement is journaled, not destroyed.** The conflict enters the
  audit trail as evidence.

## What this solves, concretely

**1. Settlement with disputes.** In a payment or escrow, a dispute means
"hold the funds, record the conflict, let the evidence resolve it" —
instead of "release to whoever pushed last" or "lock forever."

**2. Oracles with conflicting feeds.** Two price oracles disagree. The
paraconsistent gate holds the contradiction instead of averaging it
into a meaningless number — the disagreement itself is visible.

**3. Multi-agent coordination.** Autonomous agents with different
beliefs can transact without a global consensus hammer. Each
contradiction is recorded; only genuine agreement moves value.

**4. Governance with forks-but-no-fork.** Communities that disagree can
coexist — both positions are valid for their adherents, and people can
shift between them. No chain fork, no coercion.

**5. Auditable disagreement.** Because contradictions are journaled,
the *disagreement itself* becomes a first-class audit record — you can
see what was disputed, when, and by whom.

## The live implementation: ZXV on NEAR

This isn't a paper. ZXV is a live paraconsistent settlement layer on
NEAR mainnet — 110 kits in one micro contract, 136 registered
capabilities, adversarially audited to a zero-bug standard.

**The gate (the invariant, live and gas-free):**
```bash
# any client — no wallet, no gas
# conjoin {"inputs":[1,2]}       -> "FALSE"   (TRUE+FALSE=FALSE in the join)
# is_settleable {"state":"BOTH"} -> false     (a contradiction never settles)
# is_settleable {"state":"TRUE"} -> true      (agreement settles)
```

**The canonical address:** `zedec.near` — a live contract that routes
to the whole protocol:
```
zedec.near -> route()          -> the full topology
zedec.near -> protocol_meta()  -> the public record
zedec.near -> is_settleable()  -> the invariant
```

**The pieces that use the logic:**
- **The treasury** — work-based settlement: only TRUE receipts pay;
  disputes hold the funds.
- **The Venice Gas Station** — a TRUE settlement gets the protocol's 2x
  gas match; a BOTH (contradiction) forfeits the match — the drain
  shield. Disagreement has a cost, so it can't be spammed.
- **The Sovereign Decrees** — governance with forks-without-forking:
  contradictory decrees coexist, adherents shift freely.
- **The rails** — the Dragon/Phoenix/Thunderbird card networks with
  ISO/IEC 7064 check digits.

**The universal interface:** the contract teaches any client how to use
it — `ux_manifest` returns the whole schema, `ux_help` answers any
topic. The dumbest bot, the most advanced AI, and the best human all
read the same interface.

**Try it now, gas-free:**
```
# the manifest
ux_manifest on f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf
# the onboarding (a one-time 5000 YHVH grant, spendable-not-transferable)
mint_onboard {}
# the registry (136 capabilities)
vm_capabilities on 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a
```

**The quality record:** QC harness 55/55 (vs the kernel C source) · the
complete on-chain audit 24/24 · the durability stress 70/70 · the
registration audit 136/136 · gas-wall tested. Zero bugs — measured, not
claimed.

**The repositories:**
- github.com/sykomiked/zxv-stack (the 110-kit source)
- github.com/sykomiked/zxv-developer-kit (Rust + TS SDKs)
- github.com/sykomiked/ZXV-NEP-paraconsistent (the NEP file + the ledger)

---

## The takeaway

Paraconsistent logic solves the problem every distributed system
refuses: **what to do when the truth is contested.** The answer is not
to pick a side, freeze, or panic — it is to *hold the contradiction as
evidence, journal it, and settle only on genuine agreement.*

ZXV is that logic, live on NEAR — open, audited, and gas-free to read.
Build on it.

*The system is open. The tribute is 11%.*