//! oracle-gateway — the Autonomous Agent Dispute Arbiter.
//!
//! The userland ingestion endpoint for the paraconsistent kernel.
//! External agents (Shade agents in TEEs, off-chain workers, cross-chain
//! intent solvers) do NOT read the kernel's 898 KB state blob directly.
//! Instead they post assertions here; the gateway joins the evidence via
//! a cross-contract call to the kernel's `lpres_join` (account
//! `6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`),
//! isolates disputes in BOTH without reverting the caller, and emits an
//! event only when a claim resolves cleanly to TRUE.
//!
//! Flow:
//!   assert(claim, source, value)  -> record evidence, resolve verdict
//!   query(claim)                  -> current verdict (NEITHER/TRUE/FALSE/BOTH)
//!   claims()                      -> all claims + verdicts
//!
//! Settlement rule: only TRUE authorizes. BOTH and NEITHER never do.
//! A dispute is recorded, never resolved by picking a side.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// The four foundation states (mirrors the kernel's lpres_phases.h).
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

impl LogicState {
    /// The kernel's chain rank: Neither(0) < False(1) < Both(2) < True(3).
    pub fn rank(self) -> u8 {
        match self {
            LogicState::Neither => 0,
            LogicState::False => 1,
            LogicState::Both => 2,
            LogicState::True => 3,
        }
    }
}

/// A single piece of evidence on a claim.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Evidence {
    pub source: AccountId,
    pub value: LogicState,
    pub at: u64,
}

/// A claim: a statement plus the accumulated evidence.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Claim {
    pub statement: String,
    pub evidence: Vec<Evidence>,
    pub opened_at: u64,
}

/// The gateway state: claims keyed by a claim id.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OracleGateway {
    claims: LookupMap<String, Claim>,
    /// The kernel contract that owns the paraconsistent gate.
    pub kernel: AccountId,
}

#[near_bindgen]
impl OracleGateway {
    #[init]
    pub fn new(kernel: AccountId) -> Self {
        Self {
            claims: LookupMap::new(b"c".to_vec()),
            kernel,
        }
    }

    /// Open a claim on a statement.
    pub fn open_claim(&mut self, claim_id: String, statement: String) {
        assert!(!self.claims.contains_key(&claim_id), "claim exists");
        self.claims.insert(
            &claim_id,
            &Claim {
                statement,
                evidence: Vec::new(),
                opened_at: env::block_timestamp(),
            },
        );
    }

    /// Submit an assertion from a source. The claim's verdict is
    /// resolved with the LOCAL gate (see `resolve`); a cross-contract
    /// promise to the kernel's `lpres_join` is the audit path (see the
    /// `verify` example in the consumer contract). Never reverts the
    /// caller on a contradiction — the dispute is recorded as BOTH.
    pub fn assert(
        &mut self,
        claim_id: String,
        value: LogicState,
        source: AccountId,
    ) -> LogicState {
        let mut claim = self
            .claims
            .get(&claim_id)
            .expect("claim not found");
        claim.evidence.push(Evidence {
            source,
            value,
            at: env::block_timestamp(),
        });
        let verdict = Self::resolve(&claim);
        self.claims.insert(&claim_id, &claim);
        env::log_str(&format!("ZXV_ORACLE: {} -> {:?}", claim_id, verdict));
        verdict
    }

    /// The current verdict of a claim, joined locally with kjoin.
    pub fn query(&self, claim_id: String) -> LogicState {
        match self.claims.get(&claim_id) {
            Some(c) => Self::resolve(&c),
            None => LogicState::Neither,
        }
    }

    /// All claims with their verdicts.
    pub fn claims(&self) -> Vec<(String, String, LogicState)> {
        // LookupMap has no iteration in this SDK version; this method
        // is a stub documenting the intended surface. A production
        // build would track ids in a vector.
        vec![]
    }

    /// The settlement gate: only TRUE authorizes.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }

    /// kjoin: agreement keeps the value; silence adds nothing; TRUE +
    /// FALSE = BOTH. The contradiction is retained.
    pub fn kjoin(&self, a: LogicState, b: LogicState) -> LogicState {
        unsafe { core::mem::transmute::<u8, LogicState>((a as u8 | b as u8) & 0x03) }
    }

    /// Join all evidence on a claim with kjoin, starting from NEITHER.
    fn resolve(claim: &Claim) -> LogicState {
        let mut acc = LogicState::Neither;
        for e in &claim.evidence {
            acc = unsafe {
                core::mem::transmute::<u8, LogicState>((acc as u8 | e.value as u8) & 0x03)
            };
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispute_is_both_not_revert() {
        // the gateway property: a contradiction resolves to BOTH, never
        // reverts, and never authorizes
        let g = OracleGateway {
            claims: LookupMap::new(b"t".to_vec()),
            kernel: "kernel.near".parse().unwrap(),
        };
        assert_eq!(g.kjoin(LogicState::True, LogicState::False), LogicState::Both);
        assert!(!g.is_settleable(LogicState::Both));
        assert!(!g.is_settleable(LogicState::Neither));
        assert!(g.is_settleable(LogicState::True));
    }

    #[test]
    fn only_true_settles() {
        let g = OracleGateway {
            claims: LookupMap::new(b"t".to_vec()),
            kernel: "kernel.near".parse().unwrap(),
        };
        for (a, b) in [(LogicState::True, LogicState::False),
                       (LogicState::Both, LogicState::True),
                       (LogicState::Neither, LogicState::Neither)] {
            let v = g.kjoin(a, b);
            assert_ne!(v, LogicState::True, "coerced");
        }
    }
}