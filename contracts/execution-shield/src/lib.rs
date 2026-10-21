//! execution-shield — the Settlement Proxy (vault contract).
//!
//! The capital-gating subsystem: a standard NEAR vault that holds
//! operational capital and gates EVERY outbound transfer or token mint
//! strictly behind the settlement invariant
//! `lpres_is_settleable(state) == TRUE`.
//!
//! If an autonomous agent attempts a payout during an unverified or
//! disputed state (BOTH or NEITHER), the proxy HALTS the funds WITHOUT
//! aborting the agent's workflow: the call returns `false` and the
//! state stays quarantined. The invariant is enforced in the shield
//! itself (defense in depth: the kernel gate is the root, the shield is
//! the enforcement point the capital actually passes through).
//!
//! Surface:
//!   deposit()                              -> fund the vault (attached NEAR)
//!   attempt_payout(recipient, amount, state) -> TRUE-only transfer
//!   balance()                              -> vault balance
//!   is_settleable(state)                   -> the gate, exposed
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise};

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

/// The shield: a vault + the gate's enforcement point.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ExecutionShield {
    /// Only this account may authorize payouts (the autonomous agent /
    /// the OS's treasury signer).
    pub guardian: AccountId,
    /// Payouts counter (audit).
    pub payouts: u64,
    /// Blocked attempts counter (audit — disputes that were halted).
    pub blocked: u64,
}

#[near_bindgen]
impl ExecutionShield {
    #[init]
    pub fn new(guardian: AccountId) -> Self {
        Self {
            guardian,
            payouts: 0,
            blocked: 0,
        }
    }

    /// The settlement gate: only TRUE authorizes a transfer.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }

    /// Fund the vault. Any caller may deposit.
    #[payable]
    pub fn deposit(&mut self) {
        let amount = env::attached_deposit();
        assert!(amount.as_yoctonear() > 0, "deposit must be > 0");
        env::log_str(&format!("ZXV_SHIELD: deposited {}", amount));
    }

    /// The gated payout. ONLY executes when `state == TRUE`; otherwise
    /// the funds are halted and the workflow is NOT aborted — the call
    /// returns false and the attempt is counted.
    pub fn attempt_payout(&mut self, recipient: AccountId, amount: NearToken, state: LogicState) -> bool {
        // only the guardian may move capital
        assert_eq!(env::predecessor_account_id(), self.guardian, "not guardian");
        // THE SETTLEMENT INVARIANT
        if !self.is_settleable(state) {
            self.blocked += 1;
            env::log_str(&format!(
                "ZXV_SHIELD: BLOCKED payout {:?} to {} — state not TRUE, workflow preserved",
                state, recipient
            ));
            return false; // halt, do not revert the agent's workflow
        }
        let vault = env::account_balance();
        assert!(
            amount.as_yoctonear() < vault.as_yoctonear(),
            "insufficient vault balance"
        );
        self.payouts += 1;
        Promise::new(recipient).transfer(amount);
        env::log_str(&format!("ZXV_SHIELD: payout #{} executed", self.payouts));
        true
    }

    /// The vault's current balance.
    pub fn balance(&self) -> u128 {
        env::account_balance().as_yoctonear()
    }

    /// Audit counters.
    pub fn counters(&self) -> (u64, u64) {
        (self.payouts, self.blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_true_passes_the_gate() {
        let shield = ExecutionShield {
            guardian: "g.near".parse().unwrap(),
            payouts: 0,
            blocked: 0,
        };
        assert!(shield.is_settleable(LogicState::True));
        assert!(!shield.is_settleable(LogicState::Both));
        assert!(!shield.is_settleable(LogicState::Neither));
        assert!(!shield.is_settleable(LogicState::False));
    }

    #[test]
    fn dispute_halts_without_reverting() {
        // the property: a BOTH state blocks the payout decision
        let blocked = !ExecutionShield {
            guardian: "g.near".parse().unwrap(),
            payouts: 0,
            blocked: 0,
        }
        .is_settleable(LogicState::Both);
        assert!(blocked);
    }
}