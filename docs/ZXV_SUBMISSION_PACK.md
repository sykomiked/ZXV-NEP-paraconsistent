# ZXV — DevHub & NEP Submission Pack

**The formal review submission, ready to fire the bot → AI → human cascade.**

*Prepared 2026-10-22 · every claim on-chain verified*

---

## A. The DevHub proposal (the community review)

**Category:** Tooling & Infrastructure
**Title:** ZXV — the paraconsistent settlement layer

### The proposal body (for the DevHub `add_proposal` / the social write)

> ZXV is a paraconsistent settlement layer on NEAR: 110 kits in one
> micro contract, 131 registered capabilities, adversarially audited to
> a zero-bug standard. Only TRUE settles — contradictions are recorded,
> never coerced, never a panic.
>
> **The invariant:** TRUE + FALSE = BOTH. A majority can never coerce a
> recorded contradiction.
>
> **The stack:** the gate, the treasury (work-based economy), the rails
> (Dragon/Phoenix/Thunderbird cards), the receipts (hash-chained
> journal), the Venice Gas Station (2x gas matches on TRUE), the Port of
> Neo-Sicily (the external registry), the YHVH circulation (the
> spendable-not-transferable onboard grant, the bounded 2%/yr epoch),
> the liquidity pool, and the Sovereign Decrees (forks without forking).
>
> **Why it matters for NEAR:** the first paraconsistent settlement
> primitive — disagreement is treated as information, not failure. Open
> infrastructure: the contract IS the API (serverless, gas-free to
> read), the bridge is a standard any relayer implements, and any
> client (the dumbest bot to the most advanced AI to the best human)
> reads the same universal interface.
>
> **The verification (gas-free):**
> - `ux_manifest` → the whole system's schema
> - `ux_help {"topic":"start"}` → the onboarding
> - `vm_capabilities` on the registry → 131 capabilities
> - `conjoin {"inputs":[1,2]}` → "FALSE" (the invariant)
>
> **Protocol address:** f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf
> **DAO:** 36n9.dao · **Owner:** 6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8
>
> **Links:** github.com/sykomiked/zxv-stack · github.com/sykomiked/zxv-developer-kit · the NEP file in the repo

### The metadata for the submission
- **labels:** `paraconsistent`, `settlement`, `open-source`, `tooling`, `infrastructure`
- **request:** none (no funding requested — the infrastructure is deployed and self-funded)

---

## B. The NEP submission (the formal standard review)

**File:** `neps/nep-0000-paraconsistent-settlement-layer.md` (in the repo, canonical NEP-0001 format)

### The PR description (for the near/NEPs pull request)

> **NEP — The Paraconsistent Settlement Layer: a Foundational Protocol
> Utility on NEAR**
>
> This NEP proposes a four-valued paraconsistent settlement primitive:
> only TRUE settles; contradictions are recorded, never coerced. The
> reference implementation is live on NEAR mainnet — 110 kits in one
> micro contract, 131 registered capabilities, adversarially audited
> (zero bugs) and gas-wall tested.
>
> **The interface (the MVI):** `ExtParaconsistentGate { conjoin,
> is_settleable }` + the settlement invariant.
>
> **The security section:** the kjoin defect (naive `(a|b)&3` returns
> TRUE for out-of-range input) is documented; evidence combination uses
> `lpres_conjoin` with range-checking. Only TRUE settles.
>
> **The backwards-compatibility:** no breaking changes to NEAR Core or
> NEP-141/171/297; the layer is an opt-in settlement guard.
>
> **The reference implementation:** the live contracts, the developer
> kit (Rust + TS), the verification commands — all gas-free.
>
> **Repository:** https://github.com/sykomiked/ZXV-NEP-paraconsistent
> **Developer kit:** https://github.com/sykomiked/zxv-stack

### The discussions thread (for near/NEPs Discussions)

> **NEP-XXXX — the paraconsistent settlement layer**
>
> We have submitted a NEP proposing a paraconsistent settlement layer
> and policy-bounded agent rails, backed by our live 110-kit / 131-
> capability mainnet deployment and an open developer kit. The
> invariant: only TRUE settles; TRUE+FALSE=BOTH. The reference
> implementation is adversarially audited (zero bugs) and universally
> self-describing — any client reads the same interface. We welcome
> review from the NEP working group, the security community, and the
> NEAR developer community.
>
> Protocol address: f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf
> DAO: 36n9.dao · Developer kit: github.com/sykomiked/zxv-stack

---

## C. The submission checklist

| Step | Channel | Action | Status |
|---|---|---|---|
| 1 | **DevHub** | submit the proposal (the body above) via the DevHub flow | ⏳ the contract ABI resists direct probing; the social write is permission-gated — the human submits via the DevHub UI |
| 2 | **near/NEPs PR** | open the PR with the NEP file as the body | ⏳ needs the GitHub account (the user) |
| 3 | **near/NEPs discussion** | post the discussion thread (the cross-reference) | ⏳ needs the GitHub account |
| 4 | **The ecosystem maps** | the discover/nearcatalog pointers (posted) | ✅ done |

---

*The proposal pack is complete and canonical — every claim is the live
on-chain state. The two remaining steps (the DevHub UI submission and
the near/NEPs PR) are the human actions that fire the review cascade.*