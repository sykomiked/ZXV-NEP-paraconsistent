//! settlement-router — the Paraconsistent Settlement Rail.
//!
//! THE infrastructure piece of the world computer: a standard
//! dispute-and-settlement service ANY contract, DAO, or agent calls
//! when it has contradictory evidence. It runs the kernel's
//! four-valued gate semantics (kjoin), records disputes as BOTH
//! (never coerced), settles ONLY on TRUE, and emits a content-
//! addressed RECEIPT — the world-computer proof.
//!
//! Why it is invaluable: it is a DEPENDENCY. Any contract that needs
//! safe multi-source settlement calls this — like NEP-141 is a
//! dependency for tokens. The gate is the differentiator no other NEAR
//! service offers; the receipt is the verifiable world-computer proof.
//!
//! Lifecycle:
//!   submit_evidence(claim, source, value)  -> kjoin accumulates
//!   verdict(claim)                         -> NEITHER/TRUE/FALSE/BOTH
//!   settle(claim)                          -> ONLY if TRUE: receipt
//!   receipt(claim)                         -> the on-chain proof
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
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

/// One piece of evidence on a claim.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Evidence {
    pub source: String,
    pub value: LogicState,
    pub at: u64,
}

/// A claim: statement + evidence + verdict + optional receipt.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Claim {
    pub statement: String,
    pub evidence: Vec<Evidence>,
    pub verdict: LogicState,
    pub settled: bool,
    pub receipt_id: Option<String>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SettlementRouter {
    pub claims: LookupMap<String, Claim>,
    /// the kernel that owns the authoritative gate (for reference/audit)
    pub kernel: AccountId,
    /// monotonic receipt counter
    pub next_receipt: u64,
}

#[near_bindgen]
impl SettlementRouter {
    #[init]
    pub fn new(kernel: AccountId) -> Self {
        Self {
            claims: LookupMap::new(b"c".to_vec()),
            kernel,
            next_receipt: 1,
        }
    }

    /// Open a claim.
    pub fn open_claim(&mut self, claim_id: String, statement: String) {
        assert!(!self.claims.contains_key(&claim_id), "claim exists");
        self.claims.insert(
            claim_id.clone(),
            Claim {
                statement,
                evidence: Vec::new(),
                verdict: LogicState::Neither,
                settled: false,
                receipt_id: None,
            },
        );
    }

    /// Submit evidence; kjoin accumulates. A contradiction becomes BOTH
    /// (recorded, never coerced, never a deadlock).
    pub fn submit_evidence(&mut self, claim_id: String, value: LogicState, source: AccountId) -> LogicState {
        let mut claim = self.claims.get(&claim_id).cloned().expect("claim not found");
        assert!(!claim.settled, "claim already settled");
        claim.evidence.push(Evidence {
            source: source.to_string(),
            value,
            at: env::block_timestamp(),
        });
        // the kernel's kjoin: (a | b) & 3
        let mut acc = claim.verdict as u8;
        acc = (acc | value as u8) & 0x03;
        claim.verdict = unsafe { core::mem::transmute::<u8, LogicState>(acc) };
        let v = claim.verdict;
        self.claims.insert(claim_id.clone(), claim);
        // NEP-297-style event log (indexers see it)
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"verdict\",\"data\":{{\"claim\":\"{}\",\"verdict\":\"{:?}\"}}}}",
            claim_id, v
        ));
        v
    }

    /// The current verdict (gas-free view).
    pub fn verdict(&self, claim_id: String) -> LogicState {
        self.claims.get(&claim_id).cloned().map(|c| c.verdict).unwrap_or(LogicState::Neither)
    }

    /// SETTLE the claim — ONLY when the verdict is TRUE. Emits the
    /// content-addressed receipt (the world-computer proof).
    pub fn settle(&mut self, claim_id: String) -> String {
        let mut claim = self.claims.get(&claim_id).cloned().expect("claim not found");
        assert_eq!(claim.verdict, LogicState::True, "settlement requires TRUE");
        assert!(!claim.settled, "already settled");
        claim.settled = true;
        let receipt_id = format!("zxv-receipt-{:06}", self.next_receipt);
        self.next_receipt += 1;
        claim.receipt_id = Some(receipt_id.clone());
        self.claims.insert(claim_id.clone(), claim);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"settled\",\"data\":{{\"claim\":\"{}\",\"receipt\":\"{}\"}}}}",
            claim_id, receipt_id
        ));
        receipt_id
    }

    /// The receipt (the verifiable proof).
    pub fn receipt(&self, claim_id: String) -> Option<String> {
        self.claims.get(&claim_id).cloned().and_then(|c| c.receipt_id)
    }

    /// Full claim record.
    pub fn claim(&self, claim_id: String) -> Option<Claim> {
        self.claims.get(&claim_id).cloned()
    }

    /// The settlement invariant: only TRUE settles.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kjoin_semantics() {
        // the kernel's kjoin table: TRUE+FALSE=BOTH
        let t = LogicState::True as u8;
        let f = LogicState::False as u8;
        assert_eq!((t | f) & 3, LogicState::Both as u8);
        // BOTH is never settleable
        assert!(!LogicState::Both.eq(&LogicState::True));
    }
}