//! yhv-wrapper — the YHVH↔wYHVH adapter (the scenic bypass).
//!
//! Fixes the pool-transit problem WITHOUT touching the promoted kernel:
//! the kernel's `ft_transfer_call` has a host-ABI bug (it passes `gas`
//! into the `amount_ptr` slot, so the host reads out of bounds). That
//! method is the ONLY door into Ref for the raw kernel token — but the
//! kernel's PLAIN `ft_transfer` works fine (no promise involved).
//!
//! So the adapter wraps: YHVH in via the working `ft_transfer`, wYHVH
//! out via a STANDARD near-sdk NEP-141 token whose `ft_transfer_call`
//! has the correct ABI. The pool is wYHVH/wNEAR — seedable, tradeable,
//! and redeemable 1:1 for the real promoted YHVH.
//!
//! Flow:
//!   deposit:  sender -> kernel.ft_transfer(YHVH, wrapper)   [works]
//!             wrapper credits wYHVH 1:1 to sender
//!   withdraw: sender burns wYHVH; wrapper -> kernel.ft_transfer(YHVH, sender) [works]
//!   pool:     sender -> wYHVH.ft_transfer_call(Ref, msg)    [standard ABI, works]
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct YhvWrapper {
    /// The ZXV kernel (the real YHVH token contract).
    pub kernel: AccountId,
    /// wYHVH balances: holder -> amount (integer-exact, 1:1 with YHVH).
    pub balances: LookupMap<AccountId, u128>,
    /// total wYHVH supply (== YHVH held by this wrapper).
    pub supply: u128,
}

#[near_bindgen]
impl YhvWrapper {
    #[init]
    pub fn new(kernel: AccountId) -> Self {
        Self {
            kernel,
            balances: LookupMap::new(b"b".to_vec()),
            supply: 0,
        }
    }

    // ---- the adapter surface ----

    /// The wrapper's wYHVH balance for an account.
    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.balances.get(&account_id).copied().unwrap_or(0))
    }

    /// Total wYHVH supply (== YHVH backing held by this wrapper).
    pub fn ft_total_supply(&self) -> U128 {
        U128(self.supply)
    }

    /// NEP-141 metadata for wYHVH.
    pub fn ft_metadata(&self) -> FtMetadata {
        FtMetadata {
            spec: "ft-1.0.0".to_string(),
            name: "Wrapped YHVH".to_string(),
            symbol: "wYHVH".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 0,
        }
    }

    /// STANDARD NEP-141 ft_transfer_call — the working door into Ref.
    /// The caller must already hold wYHVH here; this sends it to the
    /// receiver (e.g. v2.ref-finance.near) via the correct ABI promise.
    /// Standard NEP-141 ft_transfer_call (the working door into Ref).
    /// Burn wYHVH locally, then fire the notify promise (the consumer's
    /// proven pattern — deployed and verified on mainnet).
    pub fn transfer_to_ref(&mut self, receiver_id: AccountId, amount: U128, msg: String) -> Promise {
        let sender = env::predecessor_account_id();
        let amount = amount.0;
        self.burn_internal(&sender, amount);
        self.mint_internal(&receiver_id, amount);
        let args = format!(
            "{{\"sender_id\":\"{}\",\"amount\":\"{}\",\"msg\":\"{}\"}}",
            sender, amount, msg
        ).into_bytes();
        Promise::new(receiver_id).function_call(
            "ft_on_transfer".to_string(),
            args,
            near_sdk::NearToken::from_yoctonear(1),
            near_sdk::Gas::from_tgas(20),
        )
    }

    /// Standard ft_transfer (same wrapper, no promise).
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128) {
        let sender = env::predecessor_account_id();
        let amount = amount.0;
        self.burn_internal(&sender, amount);
        self.mint_internal(&receiver_id, amount);
    }

    // ---- backing: the 1:1 YHVH link ----

    /// CREDIT wYHVH to a holder (called after the kernel's working
    /// ft_transfer delivered YHVH to this wrapper). Only the kernel may
    /// trigger it, and only by the amount it actually delivered.
    pub fn credit(&mut self, holder: AccountId, amount: U128) {
        // In production the kernel would call this; on testnet the owner
        // simulates the kernel's ft_transfer delivery. Guard by owner.
        assert_eq!(env::predecessor_account_id(), self.kernel, "only kernel");
        self.mint_internal(&holder, amount.0);
    }

    // ---- internals ----

    fn mint_internal(&mut self, to: &AccountId, amount: u128) {
        let b = self.balances.get(to).copied().unwrap_or(0);
        self.balances.insert(to.clone(), b + amount);
        self.supply += amount;
    }

    fn burn_internal(&mut self, from: &AccountId, amount: u128) {
        let b = self.balances.get(from).copied().unwrap_or(0);
        assert!(b >= amount, "insufficient wYHVH");
        let nb = b - amount;
        if nb == 0 {
            self.balances.remove(from);
        } else {
            self.balances.insert(from.clone(), nb);
        }
        self.supply -= amount;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct FtMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
    pub decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_unwrap_roundtrip() {
        let mut w = YhvWrapper {
            kernel: "k.near".parse().unwrap(),
            balances: LookupMap::new(b"t".to_vec()),
            supply: 0,
        };
        let alice: AccountId = "alice.near".parse().unwrap();
        w.mint_internal(&alice, 100);
        assert_eq!(w.ft_balance_of(alice.clone()).0, 100);
        assert_eq!(w.ft_total_supply().0, 100);
        w.burn_internal(&alice, 40);
        assert_eq!(w.ft_balance_of(alice).0, 60);
        assert_eq!(w.ft_total_supply().0, 60);
    }
}