# The Inference Invariant — mapping divergent model outputs into the lattice

**How the paraconsistent gate arbitrates decentralized AI compute.**

The core problem: floating-point inference is not bit-identical across
hardware (CUDA vs. Gaudi Synapse vs. Apple Metal produce slightly
divergent token outputs). A consensus contract that demands exact
equality reverts or disputes forever. The four-valued gate turns
divergence into *state*, not failure.

---

## 1. The mapping (evidence → lattice)

| Observation | Lattice state | Code | Meaning |
|---|---|---|---|
| All nodes agree (outputs match within tolerance) | `TRUE` | `0x01` | verified assertion — settleable |
| Nodes diverge (competing outputs, no majority) | `BOTH` | `0x03` | disputed — recorded, isolated, never settles |
| No node returned (timeout / offline / not attested) | `NEITHER` | `0x00` | unobserved — waits, never settles |
| All nodes agree the claim is false | `FALSE` | `0x02` | refuted — never settles |

The invariant: **only `TRUE` authorizes payout or finality.** `BOTH`
pauses settlement until arbitration resolves the dispute; `NEITHER`
waits for attestation; `FALSE` is permanently refused.

## 2. Tolerance & the divergence predicate

Define a deterministic equivalence on outputs:

```
match(a, b) = (a == b)                      # exact (integer/logit-identical)
            | (cosine(a, b) >= tau)         # semantic tolerance (embedding)
            | (a.logits == b.logits)        # logits-identical (sampled)
```

- Two nodes assert the same intent: `kjoin(TRUE_node1, TRUE_node2) =
  TRUE` (settleable).
- Two nodes assert different outputs: the kernel executes
  `kjoin(TRUE_node1, TRUE_node2)` over the *assertions* — both are
  TRUE assertions, but their *content* diverges, so the dispute is
  recorded as `BOTH` (per the kjoin table: TRUE + TRUE = TRUE only
  when they are the same assertion; the gate applies to the verdict on
  "do the outputs agree", which is FALSE when they differ).

**Precise formulation.** Let `agree(A, B)` be the boolean predicate
"output A and output B match within tolerance". The gate maps:

```
verdict = kjoin(agree ? TRUE : FALSE, attested ? TRUE : NEITHER)
```

- both attest + agree → `TRUE` (settle)
- both attest + disagree → `BOTH` (dispute, pause)
- one attests + other silent → `TRUE` (single attestation settles,
  per the kjoin table TRUE+NEITHER = TRUE)
- none attest → `NEITHER` (wait)
- any attestation fails verification → `FALSE` (refuse)

## 3. The dispute lifecycle (state containment)

```
        ┌──────────┐
        │  NEITHER │  waiting for attestation
        └────┬─────┘
             │ first attestation
             ▼
        ┌──────────┐        disagree        ┌──────────┐
        │   TRUE   │ ─────────────────────► │   BOTH   │  dispute open
        └────┬─────┘                        └────┬─────┘
             │ agree / arbitration              │ arbitration / TEE sig
             ▼                                  ▼
        settleable                          back to TRUE (resolved)
```

- A dispute (`BOTH`) is **recorded in the state journal as an isolated
  thread** — the system keeps running, other intents are unaffected.
- Settlement is paused **only for the disputed intent**.
- Resolution paths: (a) a third node agrees with one side (majority →
  TRUE), (b) a TEE cryptographic signature attests one output (TRUE),
  (c) timeout with no resolution → the dispute stays `BOTH`, never
  coerced.
- **Non-coercion:** no sequence of joins turns `BOTH` into `TRUE`
  without a new, authenticated attestation (see `docs/spec/algebra.md`
  §4.3 — the theorem holds on-chain, 0 coerced cells verified).

## 4. Why this is the AIVM's consensus

Traditional "exact-match" consensus is the wrong model for ML. The
paraconsistent invariant is:

1. **Never reverts on divergence** — the transaction completes, the
   dispute is isolated.
2. **Never settles on uncertainty** — `BOTH` and `NEITHER` cannot pay.
3. **Always auditable** — the dispute thread is in the journal.
4. **Hardware-agnostic** — tolerance is explicit, so CUDA/Gaudi/Metal
   divergence is *handled*, not fought.

## 5. Reference implementation

`lpres_join(kjoin, a, b) = (a | b) & 3` — the same gate, verified
on-chain (48 cells, 0 coerced). The inference invariant is a policy on
top of the gate: how evidence (node outputs) becomes truth values
(`agree`, `attested`), then the gate does the rest.

## 6. Micro-inference on-chain (the wasm path)

For models small enough to embed (BitNet 1.58 ternary, lookup-table
embeddings < 2-5 MB), the weights live in wasm linear memory and the
kernel executes the forward pass *inside the sandbox* with SIMD128
(`i8x16.add`) — deterministic, fully on-chain micro-inference. Its
output is `TRUE` by construction (the sandbox is deterministic), so the
inference invariant reduces to "did the sandbox run" — no divergence
possible. See `reference/` for the micro-inference wasm and its
benchmark.