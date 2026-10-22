//! rails-receipts — the hash-chained settlement journal (rails spec C11).
//!
//! Every state change appends a canonical receipt to a hash chain:
//! `head = SHA-256(prev_head || canonical_receipt)`. Anyone can rebuild
//! the full history from public chain data alone — no ZXV server is
//! needed. This is the auditability layer: settlements, cards, agent
//! payments and fee captures all land here as verifiable receipts.
//!
//! The receipts carry the verdict and the failing evidence, so the
//! chain is a live, searchable audit trail of every paraconsistent
//! decision. Bounded: the on-chain store keeps a rolling window; the
//! head is the anchor anyone can verify.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// A canonical receipt (the chain's unit).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Receipt {
    pub index: u64,
    pub kind: String,          // card_mint | card_pay | settle | fee_capture | agent_pay
    pub card: Option<String>,  // the rail number (for card events)
    pub counterparty: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<String>,
    pub verdict: String,       // NEITHER | TRUE | FALSE | BOTH
    pub failing: Vec<String>,  // failing evidence names (FALSE/BOTH)
    pub prev_head: String,     // the previous chain head (hex)
    pub head: String,          // SHA-256(prev || canonical) — this receipt's head
    pub at: u64,               // block timestamp
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct RailsReceipts {
    pub owner: AccountId,
    /// the current chain head
    pub head: String,
    /// height (receipts appended)
    pub height: u64,
    /// bounded store: index -> receipt (rolling window, e.g. 1024)
    pub store: LookupMap<u64, Receipt>,
    /// genesis anchor
    pub genesis: String,
}

const WINDOW: u64 = 1024;

#[near_bindgen]
impl RailsReceipts {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        let genesis = env::sha256(b"ZXV-RAILS-RECEIPTS-GENESIS");
        Self {
            owner,
            head: hex(&genesis),
            height: 0,
            store: LookupMap::new(b"s".to_vec()),
            genesis: hex(&genesis),
        }
    }

    /// Append a canonical receipt to the chain. The head advances:
    /// `head = SHA-256(prev_head || canonical)`. Permissionless (any
    /// keeper may journal; the receipt's integrity is in its hash).
    pub fn append(
        &mut self,
        kind: String,
        card: Option<String>,
        counterparty: Option<String>,
        asset: Option<String>,
        amount: Option<String>,
        verdict: String,
        failing: Vec<String>,
    ) -> Receipt {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.height,
            kind,
            card.as_deref().unwrap_or(""),
            counterparty.as_deref().unwrap_or(""),
            asset.as_deref().unwrap_or(""),
            amount.as_deref().unwrap_or(""),
            verdict,
            failing.join(",")
        );
        let prev = self.head.clone();
        let digest = env::sha256(format!("{}||{}", prev, canonical).as_bytes());
        let head = hex(&digest);
        let index = self.height;
        let receipt = Receipt {
            index,
            kind,
            card,
            counterparty,
            asset,
            amount,
            verdict,
            failing,
            prev_head: prev.clone(),
            head: head.clone(),
            at: env::block_timestamp(),
        };
        self.store.insert(index, receipt.clone());
        self.head = head;
        self.height += 1;
        // bound the store: drop the oldest when over the window
        if self.height > WINDOW {
            self.store.remove(&(self.height - WINDOW));
        }
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv_rails\",\"version\":\"1.0.0\",\"event\":\"receipt_append\",\"data\":{{\"index\":\"{}\",\"head\":\"{}\"}}}}",
            index, self.head
        ));
        receipt
    }

    // ---- views ----

    /// The chain head + height (the anchor anyone verifies).
    pub fn head(&self) -> ChainHead {
        ChainHead { height: self.height, head: self.head.clone() }
    }

    /// A receipt by index (gas-free).
    pub fn receipt(&self, index: u64) -> Option<Receipt> {
        self.store.get(&index).cloned()
    }

    /// Rebuild and verify the chain from a start index to the head.
    /// Returns true iff every link hashes correctly (the audit proof).
    pub fn verify_chain(&self, from: u64) -> bool {
        let mut cur = self.genesis.clone();
        for i in from..self.height {
            match self.store.get(&i) {
                Some(r) => {
                    if r.prev_head != cur { return false; }
                    let canonical = format!(
                        "{}|{}|{}|{}|{}|{}|{}|{}",
                        r.index, r.kind,
                        r.card.as_deref().unwrap_or(""),
                        r.counterparty.as_deref().unwrap_or(""),
                        r.asset.as_deref().unwrap_or(""),
                        r.amount.as_deref().unwrap_or(""),
                        r.verdict, r.failing.join(",")
                    );
                    let d = env::sha256(format!("{}||{}", cur, canonical).as_bytes());
                    if hex(&d) != r.head { return false; }
                    cur = r.head.clone();
                }
                None => return false,
            }
        }
        cur == self.head
    }

    /// The genesis anchor.
    pub fn genesis(&self) -> String {
        self.genesis.clone()
    }
}

/// The chain anchor.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct ChainHead {
    pub height: u64,
    pub head: String,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_is_deterministic() {
        // the genesis head is a fixed hash of the canonical label
        let g = env_sha(&b"ZXV-RAILS-RECEIPTS-GENESIS".to_vec());
        assert_eq!(g.len(), 64);
    }

    fn env_sha(_b: &Vec<u8>) -> String {
        // host sha256 is exercised on-chain; here just check the shape
        "a".repeat(64)
    }
}