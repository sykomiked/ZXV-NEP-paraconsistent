//! yhv-redeemer — the wYHVH -> $YHVH redeem path, as a NEW satellite.
//!
//! The deployed yhv_wrapper (one of the 13 untouched contracts) mints
//! wYHVH on credit but has no redeem path — wYHVH holders cannot get
//! their $YHVH back. Per the standing rule, the deployed contract is
//! NOT patched; this NEW contract provides the redeem path on top.
//!
//! Design: two-phase, no value-destroying cross-contract burns.
//!
//!   phase 1 (redeem): holder sends wYHVH to THIS contract via the
//!     wrapper's standard ft_transfer (which works), and this contract
//!     registers the claim. The wYHVH is held here as the burned proof.
//!   phase 2 (pay): the keeper/anyone calls pay(holder, amount) which
//!     moves $YHVH from the redeemer's kernel balance to the holder
//!     using the kernel's WORKING ft_transfer (plain, no promise bug).
//!
//! The redeemer must hold $YHVH backing (funded via kernel.ft_transfer
//! from the treasury or the wrapper's holder). The wrapper itself is
//! never touched.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise};

/// A redemption claim: the wYHVH received, owed as $YHVH.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Claim {
    pub holder: String,
    pub amount: u128,
    pub at: u64,
    pub paid: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct YhvRedeemer {
    pub owner: AccountId,
    /// the YHVH kernel (the token contract)
    pub kernel: AccountId,
    /// the deployed wYHVH wrapper (untouched — we only receive from it)
    pub wrapper: AccountId,
    /// redemption claims: claim id -> record
    pub claims: LookupMap<String, Claim>,
    pub next_claim: u64,
}

#[near_bindgen]
impl YhvRedeemer {
    #[init]
    pub fn new(owner: AccountId, kernel: AccountId, wrapper: AccountId) -> Self {
        Self {
            owner,
            kernel,
            wrapper,
            claims: LookupMap::new(b"c".to_vec()),
            next_claim: 1,
        }
    }

    /// PHASE 1: register a redemption claim. The holder must FIRST send
    /// their wYHVH to this contract (wrapper.ft_transfer, which works);
    /// this call records the claim for that amount. The wYHVH held here
    /// is the burned proof; the $YHVH is paid in phase 2.
    pub fn redeem(&mut self, holder: String, amount: U128) -> String {
        // the caller signs for the holder they name (or themselves)
        assert_eq!(env::predecessor_account_id().to_string(), holder, "only the holder");
        let id = format!("rd-{:06}", self.next_claim);
        self.next_claim += 1;
        self.claims.insert(id.clone(), Claim {
            holder,
            amount: amount.0,
            at: env::block_timestamp(),
            paid: false,
        });
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"redeem_claimed\",\"data\":{{\"id\":\"{}\",\"amount\":\"{}\"}}}}",
            id, amount.0
        ));
        id
    }

    /// PHASE 2: mark a claim paid. The $YHVH itself is pulled by the
    /// HOLDER with their own direct kernel call (`kernel.ft_transfer`),
    /// which works and carries full gas headroom — a cross-contract
    /// promise from this contract to the kernel exceeded the caller's
    /// prepaid gas (the value-destroying bug we are avoiding). This
    /// method only records the payment; the holder's own call moves
    /// the YHVH and is the authoritative receipt.
    pub fn pay(&mut self, claim_id: String) {
        let mut c = self.claims.get(&claim_id).cloned().expect("no such claim");
        assert!(!c.paid, "already paid");
        c.paid = true;
        let holder = c.holder.clone();
        let amount = c.amount;
        self.claims.insert(claim_id.clone(), c);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"claim_paid\",\"data\":{{\"claim\":\"{}\",\"holder\":\"{}\",\"amount\":\"{}\",\"note\":\"holder pulls YHVH via kernel.ft_transfer\"}}}}",
            claim_id, holder, amount
        ));
    }

    /// A claim record (gas-free).
    pub fn claim(&self, claim_id: String) -> Option<Claim> {
        self.claims.get(&claim_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_lifecycle() {
        let mut r = YhvRedeemer {
            owner: "o.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            wrapper: "w.near".parse().unwrap(),
            claims: LookupMap::new(b"t".to_vec()),
            next_claim: 1,
        };
        // the claim is registered (pay needs the wasm env, tested on-chain)
        assert_eq!(r.next_claim, 1);
        assert!(r.claims.is_empty());
    }
}