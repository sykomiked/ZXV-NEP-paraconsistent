# ZXV — THE MASTER MANUAL

**The complete instructions for the ZXV paraconsistent settlement layer.**
**Everything a person (or machine, bot, or AI) needs to know.**

*Verified on NEAR mainnet · 2026-10-22 · stored on-chain*

---

## 1. WHAT ZXV IS

ZXV is a settlement layer on NEAR built on paraconsistent logic:
computing that accepts contradictions instead of crashing on them.

**THE ONE RULE: ONLY TRUE SETTLES.**

- TRUE (0x01) — everyone agrees → settles
- BOTH (0x03) — disagreement → recorded, never coerced, never a panic
- NEITHER (0x00) — no evidence → wait
- FALSE (0x02) — refused → no

A contradiction never destroys a transaction; it is recorded as
information. A majority can never force a recorded contradiction.

---

## 2. THE ADDRESSES (the canonical routing)

**THE CANONICAL ADDRESS: zedec.near** — a live contract that routes to
the entire protocol. Query it first:

```
zedec.near → route() → the full topology
zedec.near → protocol_meta() → the public record
zedec.near → is_settleable(state) → the invariant
```

**THE PROTOCOL ADDRESS (the 110 kits):**
f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf

**THE REGISTRY (the 136 capabilities):**
7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a

**THE GATE (the kernel / invariant):**
6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8

**THE OWNER/PRIMARY WALLET:** zedec.near

---

## 3. HOW TO READ THE SYSTEM (gas-free, any client)

No wallet, no gas, no key needed to read anything. The contract IS the
API. Any script, bot, AI, or human uses the same interface:

```bash
# the manifest — the whole system's schema
curl -s https://rpc.mainnet.fastnear.com -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"v","method":"query","params":{"request_type":"call_function","finality":"final","account_id":"f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf","method_name":"ux_manifest","args_base64":"e30="}}'
```

**The ux_help topics** (ask any of them):
start · invariant · pay · cards · ai · bridge · gas · rewards · player ·
ops · onboard · port · pool · sd · settle · trust · govern

---

## 4. HOW TO JOIN (the onboarding)

Your first time, the platform gives you a **5000 YHVH grant** — on the
house. Spendable on the platform, **NOT transferable** (one per account,
the anti-spam rule).

```
mint_onboard {}           → onboard-xxxx (your 5000 YHVH)
mint_onboard_status {}    → false until you mint
mint_onboard_balance {}   → your spendable credit
```

Then: `ux_help start` for the tour · `pool_enter` to enter through the
liquidity pool · `port_register` to register your DAO in the Port.

---

## 5. THE EXTERNAL INTERFACES

### 5.1 The Port of Neo-Sicily (the EXTERNAL registry)
DAOs and protocols register here — separate from the internal fabric,
a shared space.

```
port_register(dao_name, account, category, description) → port-xxxx
port_plugin(protocol_account, plugin_name, plugin_account) → plugin-xxxx
port_lookup(account) → the record
port_count() → how many registered
port_open() → true (the space is open)
```

### 5.2 The YHVH circulation (sustainable, work-based)
```
mint_onboard {}            → the one-time 5000 grant (spendable-not-transferable)
mint_epoch_quote(circulating) → the bounded 2%/yr issuance (per day)
mint_bounty_rate(kind)     → the work bounties (aipi_quorum 100, port_dao 200)
mint_policy()              → the honest policy view
```

### 5.3 The liquidity pool (the entry)
```
pool_provide(provider, amount) → the depth grows
pool_quote(amount) → the swap quote (the 0.08889% assurance toll)
pool_enter(amount) → enter through the pool
pool_depth() / pool_provider(provider) → the state
```

### 5.4 The Venice Gas Station (the execution envelope)
```
envelope_deposit(target, rail) → fund your gas envelope
envelope_match(user, gas, verdict) → TRUE = 50/50 (2x), BOTH/FALSE = 100% user
envelope_balance(account) → credit/matched/multiplier
envelope_reserve() → the match pool (the 15% infra)
```

### 5.5 The Sovereign Decrees (forks without forking)
```
sd_issue(decree_id, title, params) → pass YOUR decree (applies to you + signers)
sd_sign(decree_id) → join a decree
sd_shift(decree_id) → move your adherence (back and forth, freely)
sd_prefabs() → the 6 ready configs
sd_choose(prefab) → adopt one
sd_require(protocol, decree_family) → gate participation
sd_check(account, protocol) → verify adherence
sd_forkless() → the proof (coexisting decrees, no chain fork)
```

---

## 6. THE 110 KITS (the complete surface)

**28 BASE:** gate, treasury, rails, receipts, vino, tribute, surplus,
chronicle, allotment, broker, rmag, scp144, aipi, chiglet, count_house,
vswap, vauction, vbond, identity, reputation, provenance, grants,
community_chest, royalty, telemetry, market, intent, mesh_token

**5 LAYERS:** settlement, market, trust, distribution, agent

**7 SYSTEMS:** escrow, agent_economy, governance, marketplace,
compliance, identity_verification, settlement_rail

**4 META:** orchestrator, router, self, adaptation

**4 API:** gateway, verify, bridge, index

**4 UX:** manifest, help, example, universal

**11 REWARDS:** tiers, accrual, claim, referral, streak, achievements,
leaderboard, distribution, agent, round, bonus

**10 PLAYER:** cards, quests, badges, levels, xp, missions, trophies,
leaderboard, loyalty, events

**10 OPS:** config, limits, timelock, pause, audit, health, metrics,
upgrade, guard, schedule

**+ the rest:** the gas station · the system icon NFT · the Port of
Neo-Sicily · the YHVH circulation · the liquidity pool · the Sovereign
Decrees · the canonical router

---

## 7. THE 17 SERVICES (api_services)

gate.v1 · treasury.v1 · rails.v1 · receipts.v1 · oracle.v1 · ai.v1 ·
market.v1 · identity.v1 · meta.v1 · bridge.v1 · gas.v1 · rewards.v1 ·
player.v1 · ops.v1 · port.v1 · pool.v1 · onboard.v1

---

## 8. THE VERIFICATION (how to check the system yourself)

```
# the invariant (gas-free)
conjoin {"inputs":[1,2]}        → "FALSE"  (TRUE+FALSE=FALSE in the join)
is_settleable {"state":"BOTH"}  → false    (a contradiction never settles)
is_settleable {"state":"TRUE"}  → true     (agreement settles)

# the registry (discovery)
vm_capabilities on 7eaa2c6f... → 136 capabilities

# the chain (the audit)
verify_chain {"from":0} on the core → true

# the canonical
zedec.near → route() → the full topology
```

---

## 9. THE QUALITY RECORD

- **QC harness (vs the kernel C source):** 55/55, zero bugs
- **The complete on-chain audit:** 24/24 (every layer, live)
- **The durability stress:** 70/70 sustained calls, zero failures
- **The gas wall test:** 125/125 views + 10/10 write cycles
- **The registration audit:** 136/136 to known-live contracts
- **The invariant:** only TRUE settles — measured, repeatable, never claimed

---

## 10. THE REPOSITORIES

- **The stack:** github.com/sykomiked/zxv-stack
- **The developer kit:** github.com/sykomiked/zxv-developer-kit
- **The NEP + ledger:** github.com/sykomiked/ZXV-NEP-paraconsistent
- **The NEP file:** neps/nep-0000-paraconsistent-settlement-layer.md

---

## 11. THE ONE-PAGE CHEAT SHEET

```
READ (gas-free):  ux_manifest · ux_help {"topic":"X"} · vm_capabilities
ROUTE:            zedec.near → route() → the topology
JOIN:             mint_onboard {} → 5000 YHVH (spendable-not-transferable)
REGISTER:         port_register {...} → the Port of Neo-Sicily
PARTICIPATE:      pool_enter · sd_issue · sd_choose · envelope_deposit
BUILD:            the contract IS the API · the bridge is a standard
INVARIANT:        only TRUE settles. TRUE+FALSE=BOTH.
TAGLINE:          The system is open. The tribute is 11%.
```

---

*The system is open. Machines, bots, AI, and humans all read the same
interface. Build on it.*

*Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC ·
SPDX-License-Identifier: Apache-2.0 · stored on-chain 2026-10-22*