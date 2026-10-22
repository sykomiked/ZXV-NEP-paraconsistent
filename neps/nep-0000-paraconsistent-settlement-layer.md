---
NEP: 0000 # Assigned by NEP Editors upon PR submission
Title: Paraconsistent Settlement Layer & Bounded Capabilities
Author: Michael Laurence Curzi <deal@zedec.ai>
Status: Draft
Type: Standards Track (Contract Standards / Developer Tools)
Category: Contract
Created: 2026-10-22
Requires: NEP-141, NEP-171, NEP-297
Discussions-To: https://github.com/near/NEPs/discussions/XXXX
---

# Paraconsistent Settlement Layer & Bounded Capabilities

## Summary

A four-valued paraconsistent settlement layer for NEAR: a logic kernel in
which **only `TRUE` settles**. Contradictions (`TRUE + FALSE = BOTH`) are
recorded, never coerced, and never panic. The layer is deployed as 18
registered, discoverable capabilities on NEAR mainnet: an 84-kit
consolidated core (28 base + 5 layers + 7 systems + 4 meta + 4 api +
4 ux + 11 rewards + 10 player + 10 ops + the gas station), and ships a developer kit (Rust crate + TypeScript SDK) so any
protocol, DAO, or AI agent can integrate in one import. This NEP specifies
the minimum viable interface for external contracts to interoperate with
the gate and the bounded agent rails.

## Motivation

Decentralized systems fail at the boundary of disagreement:

| Failure mode | Classical behavior | ZXV behavior |
|---|---|---|
| Two oracle feeds diverge | revert / deadlock / unfair liquidation | **held as `BOTH`** — no value moves |
| Two AI models disagree | panic or pick a side | **held** until evidence resolves |
| Two agents assert conflicting directives | undefined behavior | **recorded, never coerced** |
| Settlement ambiguity | the transaction blocks | **held; only `TRUE` settles** |

The principle of explosion means one contradictory input can destroy a
whole transaction. The NEAR ecosystem needs a settlement primitive that
treats disagreement as *information* — recorded, isolated, and resolved by
evidence — rather than as a failure. This standard provides that primitive
as composable, application-layer infrastructure, with a live reference
implementation on mainnet and an open developer kit.

## Specification

### The four-valued foundation

| State | Value | Meaning |
|---|---|---|
| `NEITHER` | `0x00` | no evidence yet |
| `TRUE` | `0x01` | attested, agreed |
| `FALSE` | `0x02` | explicitly refuted |
| `BOTH` | `0x03` | contradiction retained |

The knowledge join: `kjoin(a, b) = (a | b) & 3`. The settlement
invariant: **only `TRUE` is settleable**; `BOTH` and `NEITHER` hold;
`FALSE` refuses. `BOTH` is absorbing — a majority can never coerce a
recorded contradiction.

### Minimum Viable Interface (MVI)

Third-party contracts interoperate with the gate through the following
trait (equivalent C-ABI signatures are exported by the deployed wasm):

```rust
/// The safe evidence-combination gate.
pub trait ExtParaconsistentGate {
    /// Combine evidence with the kernel's lpres_conjoin. Every input
    /// MUST be range-checked to 0..3 before combining (the kjoin-defect
    /// fix); out-of-range input is refused, never coerced to TRUE.
    fn conjoin(&self, inputs: Vec<u8>) -> u8;

    /// The settlement invariant: only TRUE (0x01) is settleable.
    fn is_settleable(&self, state: u8) -> bool;
}

/// The bounded agent rails (card-backed, budget-scoped payments).
pub trait ExtRailsVault {
    /// Authorize a payment from a card. The verdict is computed from
    /// limits, funds, rail, and agent evidence; value moves only on TRUE.
    fn pay(&mut self, card: String, merchant: AccountId, amount: U128,
           atc: u32) -> Promise;

    /// Gas-free preview of a payment: the verdict + failing evidence.
    fn dry_run_pay(&self, card: String, merchant: AccountId,
                   amount: U128) -> (u8, Vec<String>);
}
```

The deployed contract (`zxv-core`, `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`)
exports `conjoin`, `is_settleable`, `verify_number`, `capture_toll`,
`quote_toll`, `append`, `head`, and `verify_chain`.

### The deployed fabric

`vm_capabilities()` on the world-registry
(`7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a`)
returns 103 capabilities (the 18 original names + the 84-kit surface), each
resolving to a live contract: gate, ledger,
registry, oracle, bus, shield, micro_infer, model_registry, consumer,
paraconsistent_oracle, settlement_router, yhv_wrapper, ai_verdict, and the
consolidated `zxv-core` hosting verdict_guard, treasury, yhv_redeemer,
rails_registry, and rails_receipts.

### The rail card networks

ZXV's own identifiers: 20-digit Luhn-valid rail numbers **outside
ISO/IEC 7812** (no IIN, no card-network claim, no PAN semantics):

| Network | Prefix | Inner check |
|---|---|---|
| Dragon | `8809` | ISO/IEC 7064 MOD 11,10 |
| Phoenix | `8829` | Damm |
| Thunderbird | `8849` | Verhoeff |

`verify_number(number)` returns the network or empty, check digits
byte-for-byte from the kernel's `cn_check.c`.

## Reference Implementation

- **Live on NEAR mainnet** (verified 2026-10-22): the gate at
  `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`
  (`lpres_join`) and the consolidated core at `f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf`.
- **Developer kit:** https://github.com/sykomiked/zxv-developer-kit
  (`zxv-sdk` Rust crate, `@zxv/developer-kit` TypeScript SDK)
- **Source repository:** https://github.com/sykomiked/ZXV-NEP-paraconsistent

Verify it yourself (gas-free views):

```bash
# the whole platform
near view 7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a vm_capabilities '{}'

# the gate: TRUE + FALSE = BOTH
curl -s https://rpc.mainnet.fastnear.com -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8","method_name":"lpres_join","args_base64":"AgEC"}}'

# the safe evidence gate (the kjoin-defect fix)
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf conjoin '{"inputs":[1,2]}'

# the toll (0.08889%)
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf quote_toll '{"amount":"1000000"}'

# the card networks
near view f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf verify_number '{"number":"88090016741821469134"}'
```

## Security Implications

### The kjoin defect and its fix

The naive knowledge join `(a | b) & 3` does **not** range-check inputs: on
the live gate, `kjoin(0, 5)` evaluates to `TRUE` because `(0 | 5) & 3 = 1`.
A settlement guard built on naive bitwise OR can therefore settle on
non-TRUE inputs — an unsafe-settlement vector. The standard requires
evidence combination through `lpres_conjoin` **only**, with every input
explicitly range-checked to `0..3` before combining. Out-of-range input is
refused (never coerced), and the resulting verdict can be `TRUE` only when
every input is `TRUE` (adversarially verified across the full input space:
zero invariant violations).

### Fail-closed limits

Session bounds and velocity limits halt execution when thresholds are
crossed — a bill of `10.01` against a `10.00` ceiling is declined at the
contract with nothing moved, a burst over the rate freezes the session,
and an expired session is dead. The agent cannot override, extend, or
re-sign any limit; only the holder's key can. The gate's `conjoin` is
pure (no state mutation) and the treasury's work-bounty path is
replay-refused per receipt, so there is no re-entrancy surface on the
settlement path.

### Escrow self-containment

Card escrow deposits are un-lent user balances ("spend only what you
hold"). The treasury never pools, lends, or rehypothecates them; no
yield is generated from custody.

## Alternatives

1. **Classical two-valued logic** — a divergence panics or reverts; this
   is the failure mode the standard removes. Not an alternative, the
   status quo.
2. **Fuzzy / probabilistic aggregation** — averages conflicting evidence
   into a single number, which can hide a genuine contradiction and still
   settle on a disputed value. The four-valued gate *records* the
   contradiction instead of averaging it away.
3. **Arbitrary resolution (majority vote)** — a majority can coerce a
   recorded contradiction. The standard's `BOTH`-absorbing rule prevents
   this: disputes resolve by evidence, never by votes.
4. **No gate (trust a single source)** — the status quo for most oracles;
   a single bad feed can trigger unfair liquidation. The harmonizer
   holds divergent feeds as `BOTH` instead.

## Backwards Compatibility

This standard introduces **no breaking changes** to NEAR Core runtime
APIs or existing NEP-141 / NEP-171 standards. It operates strictly at the
application/contract layer as a composable settlement guard:

- NEP-141: the `$YHVH` ledger and `wYHVH` wrapper are NEP-141-compatible;
  no change to the fungible-token standard.
- NEP-171: rail cards expose the NEP-171 read surface; `nft_transfer`
  always fails (non-transferable) — no change to the standard.
- NEP-297: every state change emits NEP-297-style events; indexers
  unaffected.
- The deployed kernel's code hash (`AiND1AEh…`) is unchanged; all
  additions are new satellite contracts or the consolidated core.

Existing contracts that do not call the gate are unaffected; the gate is
an opt-in settlement guard, not a consensus change.

---

*Repository: https://github.com/sykomiked/ZXV-NEP-paraconsistent · Developer kit: https://github.com/sykomiked/zxv-developer-kit · All on-chain claims verified against NEAR mainnet 2026-10-22.*