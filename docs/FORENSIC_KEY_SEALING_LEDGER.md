# FORENSIC KEY-SEALING LEDGER

**The permanent record of the protocol sealing — every FullAccess key deleted, every transaction hash, every contract sealed.**

*Prepared 2026-10-23 · Classification: PUBLIC (the sealing is the point; the record is the proof) · Method: `near delete-key` signed by each contract's own key*

---

## The premise

The ZXV core protocol has been **permanently sealed**: every FullAccess
key deleted from the 13 invariant-critical contracts. No party —
founder, administrator, foundation, or the ecosystem — can redeploy,
upgrade, overwrite, or delete these contracts. They are immutable
infrastructure, forever.

The **registry** (the discovery + DAO-governed extension surface)
keeps its single key so the collective governance path can operate
(bond proposals, slashing, the Port-of-Neo-Sicily policy votes).

## The sealing transactions (13 contracts, 14 keys)

| # | Contract | Keys deleted | Sealing TX | Sealed |
|---|---|---|---|---|
| 1 | `zedec.near` (canonical) | `6KkUVfaG…`, `EhUjVVa2…` | `AneCKpnUzT9mmLG2xcvg7hpSi14tjUcTxH6V8rCdwa5n` + `5L8VMSFVE4TeYrcL9ec6DUa3Snpm7jBpCmNCPoGZG5kS` | ✅ 0 keys |
| 2 | `f18889ea…` (core, 110 kits) | `HFr1d2eU…` | `2SgABNbuy24MXHpp4pLUuX3nU9n9wGqjvV7Cw5TfUSrQ` | ✅ 0 keys |
| 3 | `6ca890f7…` (gate, kernel) | `8KA7WXie…` | `(2SgABNbuy24M… batch — the core+gate sequence)` | ✅ 0 keys |
| 4 | `6a57f581…` (oracle) | `8A81Rm4y…` | `2434vppi1gDYqdB2KgYWcVf47sTv9G2ddWKcLFA5ghai` | ✅ 0 keys |
| 5 | `3c70e9c6…` (bus) | `54wKj5md…` | `23LndGkqjSXVZDTrjyV8EkaHGArp56VkwzcKDp49VUxN` | ✅ 0 keys |
| 6 | `789457e6…` (shield) | `97h7vUtC…` | `9gHcKoCnTUhPFvhxPAMgqRNSZMUkZHy2Le7hbi5iKdWe` | ✅ 0 keys |
| 7 | `86f80143…` (micro_infer) | `A5ryATPN…` | `f3dSQRLiVs9Bs5PKy5KH6qwb1XSVgMZPRnkQBnCH55z` | ✅ 0 keys |
| 8 | `77361b15…` (model_registry) | `92MNKC5Q…` | `261ugyc6cQXCGAhVZiuXdxajLvXikdQduYiXZF1M3KqU` | ✅ 0 keys |
| 9 | `8947dcb7…` (consumer) | `AEtQmoTz…` | `4czJwQsLbe62jRbEA4uVCnzr9iKnoqamnEdg6XeQH8ph` | ✅ 0 keys |
| 10 | `ea85f538…` (paraconsistent_oracle) | `GnUsiBYh…` | `9Zo5vSQgbFwrAViCAgZxokbz2d75jGxJDVrUZftRPsPX` | ✅ 0 keys |
| 11 | `8bf17701…` (settlement_router) | `ARHDvJdx…` | `DZyo2LsshhfpHaBeLKSSwSUWVvsxwUgMRSNs5sumZdmj` | ✅ 0 keys |
| 12 | `1a051ac2…` (yhv_wrapper) | `2ka7eZ2r…` | `BWEH3Hz3Us9q9EQrHeGrJw59KNPt2BxeBm3EnSvT474e` | ✅ 0 keys |
| 13 | `781fc03d…` (ai_verdict) | `95v1FsKS…` | `AdvcqUvf6oWvegJcf3VYt2a2CmMjCMTLi4aUxtuAdatG` | ✅ 0 keys |

## The verification (RPC, post-seal)

```bash
# near list-keys <account>  →  [] for all 13 sealed accounts
# the registry:            →  [1 key] (the DAO governance key)
```

All 13 verified **0 keys** via RPC immediately after the deletions.

## The state of the sealed infrastructure

| Contract | Storage (locked) | Balance | Buffer |
|---|---|---|---|
| zedec.near | 0.909 NEAR staked | 1.409 | 0.500 |
| core (110 kits) | 4.840 | 5.340 | 0.500 |
| gate (kernel) | 9.504 | 10.004 | 0.500 |
| oracle | 1.392 | 1.891 | 0.500 |
| bus | 1.457 | 1.957 | 0.500 |
| shield | 1.275 | 1.775 | 0.500 |
| micro_infer | 1.048 | 1.548 | 0.500 |
| model_registry | 1.348 | 1.849 | 0.500 |
| consumer | 1.212 | 1.712 | 0.500 |
| paraconsistent_oracle | 1.475 | 1.975 | 0.500 |
| settlement_router | 1.513 | 1.999 | 0.486 |
| yhv_wrapper | 1.380 | 1.697 | 0.317 |
| ai_verdict | 1.404 | 1.999 | 0.595 |

**Every sealed contract holds a ≥0.3 NEAR free buffer** — the
storage-staking requirement is satisfied with headroom, so no sealed
contract can lock on storage growth. (The registry, which keeps its
key, is top-up-able if its code grows again.)

## The governance surface (kept open by design)

| Entity | Role | Key |
|---|---|---|
| The registry | the DAO-governed extension surface | **1 key retained** |
| `illumaheart.near` | the foundation (Port seat, policy voter) | its own key |
| `jollydragonroger.near` | the administrator (Port seat, policy voter) | its own key |

The core protocol is **closed and immutable**; governance and policy
are **open to every Port-registered entity** — the collective decides
(activate >50%, slash ≥2/3, parameter unanimous; only TRUE settles).

---

*The code is autonomous. The state machine is closed. The protocol belongs to the ecosystem.*

*Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC · SPDX-License-Identifier: Apache-2.0*