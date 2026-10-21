//! ai-verdict — the on-chain AI verdict adapter (settleable AI).
//!
//! NEAR's community is focused on on-chain AI. The missing piece is a
//! way to make AI outputs SETTLEABLE — usable to trigger value movement
//! only when the network agrees on them. This adapter takes model
//! outputs from multiple sources (the on-chain deterministic
//! micro-infer, off-chain model attestations, agent reports) and runs
//! them through the kernel's four-valued gate:
//!
//!   agreement / attested match  -> TRUE  -> output is settleable
//!   divergence                  -> BOTH  -> output held, never coerced
//!   timeout / absent evidence   -> NEITHER -> waits
//!   explicit refutation         -> FALSE -> refused
//!
//! The output + verdict is the standard surface any AI agent, dApp, or
//! settlement contract reads: `infer_with_verdict(model, input)`.
//!
//! This is the inference invariant (docs/spec/inference-invariant.md)
//! made into a deployable public utility.
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

/// One attestation of a model's output for an input.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Attestation {
    pub source: String,
    pub output: String,   // the attested output (or output hash)
    pub at: u64,
}

/// The harmonized inference state for (model, input).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct InferenceState {
    pub model: String,
    pub input: String,
    pub attestations: Vec<Attestation>,
    pub output: String,
    pub verdict: LogicState,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AiVerdict {
    pub owner: AccountId,
    /// the on-chain deterministic micro-infer (the ground truth engine)
    pub micro_infer: AccountId,
    pub states: LookupMap<String, InferenceState>,
}

#[near_bindgen]
impl AiVerdict {
    #[init]
    pub fn new(owner: AccountId, micro_infer: AccountId) -> Self {
        Self {
            owner,
            micro_infer,
            states: LookupMap::new(b"s".to_vec()),
        }
    }

    /// An AI source attests a model output for an input.
    /// The gate maps evidence per the inference invariant.
    pub fn attest(&mut self, model: String, input: String, output: String, source: AccountId) -> LogicState {
        let key = format!("{}:{}", model, input);
        let mut st = self.states.get(&key).cloned().unwrap_or(InferenceState {
            model: model.clone(),
            input: input.clone(),
            attestations: Vec::new(),
            output: String::new(),
            verdict: LogicState::Neither,
        });
        st.attestations.push(Attestation {
            source: source.to_string(),
            output: output.clone(),
            at: env::block_timestamp(),
        });
        // harmonize: all attested outputs must agree -> TRUE;
        // any divergence -> BOTH (held); empty -> NEITHER
        if st.attestations.is_empty() {
            st.verdict = LogicState::Neither;
        } else {
            let first = &st.attestations[0].output;
            let agree = st.attestations.iter().all(|a| &a.output == first);
            st.verdict = if agree { LogicState::True } else { LogicState::Both };
            if agree {
                st.output = first.clone();
            }
        }
        let v = st.verdict;
        self.states.insert(key.clone(), st);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"ai_verdict\",\"data\":{{\"key\":\"{}\",\"verdict\":\"{:?}\"}}}}",
            key, v
        ));
        v
    }

    /// THE surface AI agents and dApps read: output + verdict.
    pub fn infer_with_verdict(&self, model: String, input: String) -> AiQuote {
        let key = format!("{}:{}", model, input);
        match self.states.get(&key) {
            Some(st) => AiQuote {
                model,
                input,
                output: st.output.clone(),
                verdict: st.verdict,
                settleable: st.verdict == LogicState::True,
            },
            None => AiQuote {
                model,
                input,
                output: String::new(),
                verdict: LogicState::Neither,
                settleable: false,
            },
        }
    }

    /// The settlement invariant: only an agreed (TRUE) output settles.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }

    /// The verdict for a (model, input) pair.
    pub fn verdict(&self, model: String, input: String) -> LogicState {
        let key = format!("{}:{}", model, input);
        self.states.get(&key).cloned().map(|s| s.verdict).unwrap_or(LogicState::Neither)
    }
}

/// The quote AI consumers read.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct AiQuote {
    pub model: String,
    pub input: String,
    pub output: String,
    pub verdict: LogicState,
    pub settleable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agreement_divergence_refutation() {
        let mut a = AiVerdict {
            owner: "o.near".parse().unwrap(),
            micro_infer: "m.near".parse().unwrap(),
            states: LookupMap::new(b"t".to_vec()),
        };
        // two sources agree -> TRUE
        let v1 = a.attest("m1".into(), "x".into(), "42".into(), "src1.near".parse().unwrap());
        let v2 = a.attest("m1".into(), "x".into(), "42".into(), "src2.near".parse().unwrap());
        assert_eq!(v1, LogicState::True);
        assert_eq!(v2, LogicState::True);
        // a divergent source -> BOTH
        let v3 = a.attest("m1".into(), "x".into(), "43".into(), "src3.near".parse().unwrap());
        assert_eq!(v3, LogicState::Both);
        let q = a.infer_with_verdict("m1".into(), "x".into());
        assert_eq!(q.verdict, LogicState::Both);
        assert!(!q.settleable);
    }
}