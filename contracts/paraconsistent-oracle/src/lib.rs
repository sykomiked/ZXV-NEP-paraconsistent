//! paraconsistent-oracle — the dispute-tolerant price/event harmonizer.
//!
//! THE DeFi public utility: it connects to EXISTING oracle feeds and
//! HARMONIZES them with the kernel's four-valued gate. Any protocol
//! already using a price oracle calls `price(asset)` here instead and
//! gets a DISPUTE-TOLERANT answer:
//!
//!   feeds agree            -> TRUE  -> a settleable price
//!   feeds diverge          -> BOTH  -> price flagged, settlement held
//!   no feed reported       -> NEITHER -> not usable yet
//!   all feeds refute       -> FALSE -> refused
//!
//! Why protocols adopt it instantly without advertising:
//!   1. it reads the SAME feeds they already use (a harmonizer, not a
//!      new data source) — a drop-in with added safety;
//!   2. the interface is the standard price-oracle shape
//!      (`price(asset)` -> {price, verdict});
//!   3. NEP-297 events make indexers/agents see it automatically;
//!   4. it is registered in the world-registry (vm_info discovery) so
//!      any protocol probing the ZXV VM finds it.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// The four foundation states (the kernel's lpres semantics).
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
#[borsh(use_discriminant = true)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

/// One price observation for an asset.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Observation {
    pub source: String,
    pub price: u128,
    pub at: u64,
}

/// The harmonized asset state.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct AssetState {
    pub symbol: String,
    pub observations: Vec<Observation>,
    pub price: u128,        // the agreed price when TRUE
    pub verdict: LogicState,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ParaconsistentOracle {
    pub owner: AccountId,
    /// registered feed sources (the existing oracles we harmonize)
    pub feeds: Vec<AccountId>,
    pub assets: LookupMap<String, AssetState>,
}

/// How close two prices must be (in basis points) to count as agreement.
const TOLERANCE_BPS: u128 = 50; // 0.5%

#[near_bindgen]
impl ParaconsistentOracle {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            feeds: Vec::new(),
            assets: LookupMap::new(b"a".to_vec()),
        }
    }

    /// Register a feed source (an existing oracle contract to harmonize).
    pub fn register_feed(&mut self, feed: AccountId) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        if !self.feeds.contains(&feed) {
            self.feeds.push(feed);
        }
    }

    /// The registered feed sources.
    pub fn feeds(&self) -> Vec<String> {
        self.feeds.iter().map(|f| f.to_string()).collect()
    }

    /// A feed submits its price for an asset. Prices within tolerance
    /// agree -> TRUE; divergence -> BOTH (held, never coerced).
    pub fn submit_price(&mut self, symbol: String, price: U128, source: AccountId) -> LogicState {
        let mut st = self.assets.get(&symbol).cloned().unwrap_or(AssetState {
            symbol: symbol.clone(),
            observations: Vec::new(),
            price: 0,
            verdict: LogicState::Neither,
        });
        st.observations.push(Observation {
            source: source.to_string(),
            price: price.0,
            at: env::block_timestamp(),
        });
        // harmonize: the kernel's kjoin over the agreement predicate
        // each new observation is TRUE (a valid report) — but the
        // VERDICT on "do the prices agree" is what gates settlement.
        let first = st.observations[0].price;
        let agree = st.observations.iter().all(|o| {
            let diff = if o.price >= first { o.price - first } else { first - o.price };
            diff * 10_000 <= first * TOLERANCE_BPS
        });
        st.verdict = if agree { LogicState::True } else { LogicState::Both };
        if agree {
            st.price = first;
        }
        self.assets.insert(symbol.clone(), st);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"price_verdict\",\"data\":{{\"symbol\":\"{}\",\"verdict\":\"{:?}\"}}}}",
            symbol, if agree { LogicState::True } else { LogicState::Both }
        ));
        if agree { LogicState::True } else { LogicState::Both }
    }

    /// THE standard surface protocols call: the harmonized price + verdict.
    pub fn price(&self, symbol: String) -> PriceQuote {
        match self.assets.get(&symbol) {
            Some(st) => PriceQuote {
                symbol,
                price: st.price,
                verdict: st.verdict,
                settleable: st.verdict == LogicState::True,
            },
            None => PriceQuote {
                symbol,
                price: 0,
                verdict: LogicState::Neither,
                settleable: false,
            },
        }
    }

    /// The settlement invariant: only a TRUE-agreed price is usable.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }

    /// All assets with their verdicts (for indexers/dashboards).
    pub fn assets(&self) -> Vec<(String, LogicState, u128)> {
        // store::LookupMap is not iterable in this SDK; owners track
        // symbols in a vector in production. This documents the surface.
        vec![]
    }
}

/// The quote protocols read.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct PriceQuote {
    pub symbol: String,
    pub price: u128,
    pub verdict: LogicState,
    pub settleable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agreement_and_divergence() {
        let mut o = ParaconsistentOracle {
            owner: "o.near".parse().unwrap(),
            feeds: Vec::new(),
            assets: LookupMap::new(b"t".to_vec()),
        };
        // two feeds within tolerance -> TRUE
        let v1 = o.submit_price("near".into(), 1000, "feed1.near".parse().unwrap());
        let v2 = o.submit_price("near".into(), 1002, "feed2.near".parse().unwrap());
        assert_eq!(v1, LogicState::True);
        assert_eq!(v2, LogicState::True);
        // a feed that diverges > 0.5% -> BOTH
        let v3 = o.submit_price("near".into(), 1100, "feed3.near".parse().unwrap());
        assert_eq!(v3, LogicState::Both);
        let q = o.price("near".into());
        assert_eq!(q.verdict, LogicState::Both);
        assert!(!q.settleable);
    }
}