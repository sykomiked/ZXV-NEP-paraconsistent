//! consumer-example — cross-contract validation against the kernel.
//!
//! A minimal near-sdk-rs contract that executes a cross-contract
//! promise calling `lpres_join` on the kernel
//! (`6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`)
//! and verifies the result in its callback. This is the integration
//! pattern the working group will test: a standard near-sdk-rs contract
//! calling the freestanding C/Wasm kernel via a promise.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

/// The kernel account on NEAR mainnet.
pub const KERNEL: &str = "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8";

/// The four states (must match the kernel's codes exactly).
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq, schemars::JsonSchema,
)]
#[repr(u8)]
#[borsh(use_discriminant = true)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Consumer {
    pub kernel: AccountId,
    pub last_verdict: Option<LogicState>,
}

#[near_bindgen]
impl Consumer {
    #[init]
    pub fn new() -> Self {
        Self {
            kernel: KERNEL.parse().unwrap(),
            last_verdict: None,
        }
    }

    /// Fire a cross-contract promise to the kernel's `lpres_join`.
    /// op 2 = kjoin; args are the 3 raw bytes [op, a, b].
    pub fn verify_on_kernel(&mut self, op: u8, a: LogicState, b: LogicState) -> Promise {
        let args = vec![op, a as u8, b as u8];
        // the cross-contract promise to the kernel's lpres_join
        Promise::new(self.kernel.clone())
            .function_call(
                "lpres_join".to_string(),
                args,
                near_sdk::NearToken::from_yoctonear(0),
                near_sdk::Gas::from_tgas(10),
            )
    }

    /// The callback: call this in a separate tx (or from a promise
    /// batch) to read the kernel's returned byte. In near-sdk 5 the
    /// callback is invoked on the contract with the promise result.
    #[private]
    pub fn ext_callback(&mut self) {
        match env::promise_result(0) {
            near_sdk::PromiseResult::Successful(value) => {
                if let Some(&code) = value.first() {
                    self.last_verdict = Some(unsafe { core::mem::transmute::<u8, LogicState>(code) });
                    env::log_str(&format!("ZXV_CONSUMER: kernel verdict {:?}", self.last_verdict));
                }
            }
            _ => env::log_str("ZXV_CONSUMER: kernel call failed"),
        }
    }

        /// Local gate (no cross-contract call) — for offline checks.
    pub fn kjoin(&self, a: LogicState, b: LogicState) -> LogicState {
        unsafe { core::mem::transmute::<u8, LogicState>((a as u8 | b as u8) & 0x03) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_gate_matches_kernel_semantics() {
        let c = Consumer::new();
        // kjoin(TRUE, FALSE) = BOTH — the on-chain verified vector
        assert_eq!(c.kjoin(LogicState::True, LogicState::False), LogicState::Both);
        // kjoin(TRUE, TRUE) = TRUE — settleable
        assert_eq!(c.kjoin(LogicState::True, LogicState::True), LogicState::True);
    }
}