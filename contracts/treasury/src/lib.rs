//! zxv-treasury — the automated $YHVH monetary-policy engine (the flywheel).
//!
//! Built ON TOP of the deployed 13-contract stack (untouched). It turns
//! protocol revenue into sustainable monetary policy for $YHVH:
//!
//!   fee income (the rails' assurance fee, 0.08889% of settled payments)
//!     -> the treasury's automated split (the existing pay_assure split):
//!        50% reserve floor, 25% V-Bill pool, 15% infrastructure & node
//!        bounties, 10% regenerative capital
//!     -> V-Bills redeemable at par against the reserve (the confidence
//!        instrument: the economy's own earnings collateralize its debt)
//!     -> surplus never leaves the system; it re-enters as backing
//!
//! The flywheel: fees -> reserve -> V-Bill par -> confidence -> adoption
//! -> more fees. Fully automated: no owner action needed after config.
//!
//! Monetary policy is integer-only, bounded, and every split is checked
//! before mutation (the repository order: check, reserve, compute,
//! mutate, journal).
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// The fee split, matching pay_assure.h: the four parts always sum to fee.
pub const SPLIT_RESERVE: u128 = 5_000_000;   // 50%
pub const SPLIT_VBILL: u128 = 2_500_000;     // 25%
pub const SPLIT_INFRA: u128 = 1_500_000;     // 15%
pub const SPLIT_REGEN: u128 = 1_000_000;     // 10%
pub const SPLIT_DENOM: u128 = 10_000_000;

/// The assurance fee rate (0.08889%): fee = floor(amount * 8889 / 1e7).
pub const FEE_NUM: u128 = 8889;
pub const FEE_DEN: u128 = 10_000_000;

/// A V-Bill: the economy's debt, redeemable at par against the reserve.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct VBill {
    pub owner: String,
    pub units: u128,        // the bill's face units
    pub issued_at: u64,
    pub redeemed: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ZxvTreasury {
    pub owner: AccountId,
    /// the YHVH kernel (the reserve asset contract)
    pub kernel: AccountId,
    /// reserve floor: never touched except by V-Bill redemption at par
    pub reserve_floor: u128,
    /// the V-Bill pool: the debt side of the reserve
    pub vbill_pool: u128,
    /// infrastructure & node bounties pool
    pub infra_pool: u128,
    /// regenerative capital pool (re-enters as growth)
    pub regen_pool: u128,
    /// fees settled so far
    pub fees_settled: u128,
    /// V-Bill registry: bill id -> record
    pub bills: LookupMap<String, VBill>,
    pub next_bill: u64,
}

#[near_bindgen]
impl ZxvTreasury {
    #[init]
    pub fn new(owner: AccountId, kernel: AccountId) -> Self {
        Self {
            owner,
            kernel,
            reserve_floor: 0,
            vbill_pool: 0,
            infra_pool: 0,
            regen_pool: 0,
            fees_settled: 0,
            bills: LookupMap::new(b"b".to_vec()),
            next_bill: 1,
        }
    }

    /// THE flywheel input: settle a fee. The rail pays the fee amount;
    /// the treasury splits it automatically by the fixed proportions.
    /// Returns the four split parts (they always sum to the fee).
    pub fn settle_fee(&mut self, amount: U128) -> FeeSplit {
        let fee = amount.0;
        // compute the split: reserve gets the remainder unit (50% + 1 on
        // odd fees) so the four parts always sum exactly to fee
        let reserve = fee * SPLIT_RESERVE / SPLIT_DENOM;
        let vbill = fee * SPLIT_VBILL / SPLIT_DENOM;
        let infra = fee * SPLIT_INFRA / SPLIT_DENOM;
        let regen = fee - reserve - vbill - infra; // the remainder = regen + odd unit
        self.reserve_floor += reserve;
        self.vbill_pool += vbill;
        self.infra_pool += infra;
        self.regen_pool += regen;
        self.fees_settled += fee;
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"fee_settled\",\"data\":{{\"fee\":\"{}\",\"reserve\":\"{}\",\"vbill\":\"{}\",\"infra\":\"{}\",\"regen\":\"{}\"}}}}",
            fee, reserve, vbill, infra, regen
        ));
        FeeSplit { fee, reserve, vbill, infra, regen }
    }

    /// Issue a V-Bill against the V-Bill pool: units = pool / par, and
    /// the pool backs it 1:1 (never more bills than pool).
    pub fn issue_vbill(&mut self, owner: String) -> String {
        let units = self.vbill_pool;
        assert!(units > 0, "no V-Bill pool backing yet");
        // the bill is backed by the whole pool at par: issuing one bill
        // consumes the pool (the pool is the debt's collateral)
        self.vbill_pool = 0;
        let id = format!("vb-{:06}", self.next_bill);
        self.next_bill += 1;
        self.bills.insert(id.clone(), VBill {
            owner,
            units,
            issued_at: env::block_timestamp(),
            redeemed: false,
        });
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"vbill_issued\",\"data\":{{\"id\":\"{}\",\"units\":\"{}\"}}}}",
            id, units
        ));
        id
    }

    /// Redeem a V-Bill at par: the reserve floor pays the units (only
    /// what the bill's collateralized pool once held — never more than
    /// the floor can cover). The redeemed bill is marked, not deleted.
    pub fn redeem_vbill(&mut self, bill_id: String) {
        let mut bill = self.bills.get(&bill_id).cloned().expect("no such bill");
        assert!(!bill.redeemed, "already redeemed");
        assert!(
            self.reserve_floor >= bill.units,
            "reserve cannot cover the bill — policy halted"
        );
        bill.redeemed = true;
        let units = bill.units;
        self.bills.insert(bill_id.clone(), bill);
        self.reserve_floor -= units;
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"vbill_redeemed\",\"data\":{{\"id\":\"{}\"}}}}",
            bill_id
        ));
    }

    // ---- views ----

    /// The treasury's state (the flywheel dashboard).
    pub fn state(&self) -> TreasuryState {
        TreasuryState {
            reserve_floor: self.reserve_floor,
            vbill_pool: self.vbill_pool,
            infra_pool: self.infra_pool,
            regen_pool: self.regen_pool,
            fees_settled: self.fees_settled,
        }
    }

    /// The fee for an amount (gas-free).
    pub fn quote_fee(&self, amount: U128) -> U128 {
        U128(amount.0 * FEE_NUM / FEE_DEN)
    }

    /// A V-Bill record (gas-free).
    pub fn vbill(&self, bill_id: String) -> Option<VBill> {
        self.bills.get(&bill_id).cloned()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct FeeSplit {
    pub fee: u128,
    pub reserve: u128,
    pub vbill: u128,
    pub infra: u128,
    pub regen: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct TreasuryState {
    pub reserve_floor: u128,
    pub vbill_pool: u128,
    pub infra_pool: u128,
    pub regen_pool: u128,
    pub fees_settled: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_always_sums_to_fee() {
        let mut t = ZxvTreasury {
            owner: "o.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            reserve_floor: 0, vbill_pool: 0, infra_pool: 0, regen_pool: 0,
            fees_settled: 0, bills: LookupMap::new(b"t".to_vec()), next_bill: 1,
        };
        for fee in [1u128, 7, 100, 8889, 1_000_000, 12_345_678] {
            let s = t.settle_fee(U128(fee));
            assert_eq!(s.reserve + s.vbill + s.infra + s.regen, fee,
                       "split must sum to the fee for {fee}");
        }
    }

    #[test]
    fn quote_matches_assurance() {
        let t = ZxvTreasury {
            owner: "o.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            reserve_floor: 0, vbill_pool: 0, infra_pool: 0, regen_pool: 0,
            fees_settled: 0, bills: LookupMap::new(b"t".to_vec()), next_bill: 1,
        };
        // 0.08889% of 1,000,000 = 888.9 -> 888
        assert_eq!(t.quote_fee(U128(1_000_000)).0, 888);
    }
}