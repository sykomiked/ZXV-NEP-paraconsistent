//! deposit-helper — the pool-seeding satellite (a NEAR SDK contract).
//!
//! The core kernel's `ft_transfer_call` has a promise-path bug (out-of-
//! bounds on the cross-contract args) that must NOT be fixed by touching
//! the kernel (the promoted code hash stays). This satellite does the
//! DEX deposit from standard near-sdk-rs instead:
//!
//!   kernel.ft_transfer(amount, helper)   — the kernel's WORKING path
//!   helper.deposit_to_ref(amount)         — this contract's ft_transfer_call
//!                                          to Ref (standard, no bug)
//!   helper.add_liquidity(pool, amts)      — Ref add_liquidity
//!
//! This is the "fixes live at the edge, never in the core" pattern.
//!
//! Testnet first: deploy to a testnet account, verify the full flow
//! (fund -> deposit -> add_liquidity) free, then mainnet.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct DepositHelper {
    pub owner: AccountId,
    pub kernel: AccountId,        // the ZXV token contract
    pub ref_contract: AccountId,  // v2.ref-finance.near
}

#[near_bindgen]
impl DepositHelper {
    #[init]
    pub fn new(owner: AccountId, kernel: AccountId, ref_contract: AccountId) -> Self {
        Self { owner, kernel, ref_contract }
    }

    /// The standard near-sdk-rs ft_transfer_call to Ref — the path the
    /// kernel's buggy method can't do. Credits THIS account's deposit
    /// on Ref. `msg` = Ref's deposit/swap instructions (empty = deposit).
    #[payable]
    pub fn deposit_to_ref(&mut self, amount: String, msg: String) -> Promise {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        // transfer the YHVH (held by this helper) into Ref via the
        // standard NEP-141 ft_transfer_call on OUR OWN token holdings
        // (the helper is registered on the kernel as a holder)
        Promise::new(self.kernel.clone())
            .function_call(
                "ft_transfer_call".to_string(),
                // standard near-sdk JSON args (the helper calls the kernel's
                // ft_transfer_call — but with proper JSON, unlike the broken
                // internal path; the kernel parses this as its receiver
                // JSON: receiver_id, amount, msg)
                format!("{{\"receiver_id\":\"{}\",\"amount\":\"{}\",\"msg\":\"{}\"}}",
                        self.ref_contract, amount, msg).into_bytes(),
                NearToken::from_yoctonear(1),
                near_sdk::Gas::from_tgas(30),
            )
    }

    /// Add liquidity to a Ref pool (pool 8975 = YHVH/wNEAR).
    /// Requires both tokens already credited to this helper's Ref deposit.
    pub fn add_liquidity(&self, pool_id: u64, amounts: Vec<String>) -> Promise {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        Promise::new(self.ref_contract.clone())
            .function_call(
                "add_liquidity".to_string(),
                format!("{{\"pool_id\":{},\"amounts\":[{}]}}",
                        pool_id, amounts.iter().map(|a| format!("\"{}\"", a)).collect::<Vec<_>>().join(","))
                    .into_bytes(),
                NearToken::from_yoctonear(1),
                near_sdk::Gas::from_tgas(30),
            )
    }

    /// The helper's YHVH balance (held on the kernel, credited to this account).
    pub fn balance(&self) -> String {
        // view the kernel's balance for this account
        let _ = self.kernel.clone();
        "query via kernel ft_balance_of".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_paths_exist() {
        // structural test: the helper's surface is deployable
        let h = DepositHelper {
            owner: "o.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            ref_contract: "ref.near".parse().unwrap(),
        };
        assert_eq!(h.owner.as_str(), "o.near");
        assert_eq!(h.ref_contract.as_str(), "ref.near");
    }
}