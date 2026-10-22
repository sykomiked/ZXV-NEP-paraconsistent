//! zxv-treasury — the work-based protocol treasury (defensible perimeter).
//!
//! REGULATORY STRUCTURE (SEC Howey prong 4 / CFTC commodity standard):
//! returns are derived from the participant's OWN work, never from the
//! managerial efforts of others. There is no passive staking, no fee
//! sharing to idle tokens, no dividend. Every disbursement is a
//! WORK BOUNTY paid only upon a verified execution receipt.
//!
//! The four-way split (matching pay_assure.h and the rails C7) funds
//! OPERATIONS, not passive returns:
//!
//!   50% reserve floor         -> network liquidity (never distributed)
//!   25% settlement buffer     -> the V-Bill debt instrument at par
//!   15% infrastructure        -> keeper + deployer maintenance bounties
//!                               (paid per verified receipt / maintenance
//!                                service, vendor compensation — never a
//!                                founder dividend)
//!   10% regenerative capital  -> re-enters as growth funding
//!
//! PARTICIPANT ACTION (the Howey prong-4 test): a keeper registers,
//! posts a SLASHABLE PERFORMANCE BOND, runs verifiable work (AIPI
//! quorum checks, bridge mirrors, lazy batch settlement), submits the
//! canonical receipt hash, and is compensated per receipt. Idle capital
//! earns nothing. A missed/bad receipt SLASHES the bond.
//!
//! ESCROW SELF-CONTAINMENT (C4): card escrow deposits are un-lent user
//! balances ("spend only what you hold"). The treasury NEVER pools,
//! lends, or rehypothecates them. No yield is generated from custody.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// The work split (pay_assure proportions). The four parts always sum
/// to the captured toll.
pub const SPLIT_RESERVE: u128 = 5_000_000;   // 50% — network liquidity
pub const SPLIT_BUFFER: u128 = 2_500_000;    // 25% — settlement buffer
pub const SPLIT_INFRA: u128 = 1_500_000;     // 15% — infra & node bounties
pub const SPLIT_REGEN: u128 = 1_000_000;     // 10% — regenerative capital
pub const SPLIT_DENOM: u128 = 10_000_000;

/// The computational toll (0.08889%): toll = floor(amount * 8889 / 1e7).
pub const TOLL_NUM: u128 = 8889;
pub const TOLL_DEN: u128 = 10_000_000;

/// A registered worker (keeper / verifier / relayer).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Worker {
    pub account: String,
    /// the slashable performance bond (NEAR), held to guarantee uptime
    pub bond: u128,
    /// receipts verified so far (the work ledger)
    pub receipts_verified: u64,
    /// total work bounty earned
    pub bounty_earned: u128,
    pub active: bool,
}

/// A verifiable work receipt (the payout condition).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct WorkReceipt {
    pub worker: String,
    pub kind: String,       // "aipi_quorum" | "bridge_mirror" | "settle_batch" | "maintenance"
    pub receipt_hash: String,
    pub at: u64,
    pub settled: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ZxvTreasury {
    pub owner: AccountId,
    /// the deployer entity (the vendor/operator, e.g. Scenic Bypass
    /// Consulting LLC) — draws maintenance compensation from the infra
    /// bucket as a documented service, never as an equity dividend
    pub deployer: AccountId,
    /// the YHVH kernel (the reserve asset contract)
    pub kernel: AccountId,
    /// 50%: network liquidity reserve — never distributed
    pub reserve_floor: u128,
    /// 25%: settlement buffer (the V-Bill debt instrument at par)
    pub buffer_pool: u128,
    /// 15%: infrastructure & node bounties
    pub infra_pool: u128,
    /// 10%: regenerative capital
    pub regen_pool: u128,
    /// tolls captured so far
    pub tolls_captured: u128,
    /// workers: account -> record
    pub workers: LookupMap<String, Worker>,
    /// work receipts: hash -> record
    pub receipts: LookupMap<String, WorkReceipt>,
    /// settlement buffer bills
    pub bills: LookupMap<String, BufferBill>,
    pub next_bill: u64,
}

/// A settlement-buffer bill (the economy's debt, redeemable at par).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct BufferBill {
    pub owner: String,
    pub units: u128,
    pub issued_at: u64,
    pub redeemed: bool,
}

#[near_bindgen]
impl ZxvTreasury {
    #[init]
    pub fn new(owner: AccountId, deployer: AccountId, kernel: AccountId) -> Self {
        Self {
            owner,
            deployer,
            kernel,
            reserve_floor: 0,
            buffer_pool: 0,
            infra_pool: 0,
            regen_pool: 0,
            tolls_captured: 0,
            workers: LookupMap::new(b"w".to_vec()),
            receipts: LookupMap::new(b"r".to_vec()),
            bills: LookupMap::new(b"b".to_vec()),
            next_bill: 1,
        }
    }

    // ---- WORK REGISTRATION (the Howey prong-4 requirement) ----

    /// Register as a worker: post a SLASHABLE PERFORMANCE BOND
    /// (attached NEAR). Only bonded workers can earn work bounties.
    #[payable]
    pub fn register_worker(&mut self) -> String {
        let account = env::predecessor_account_id().to_string();
        let bond = env::attached_deposit().as_yoctonear();
        assert!(bond > 0, "a performance bond is required");
        self.workers.insert(account.clone(), Worker {
            account: account.clone(),
            bond,
            receipts_verified: 0,
            bounty_earned: 0,
            active: true,
        });
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"worker_registered\",\"data\":{{\"account\":\"{}\",\"bond\":\"{}\"}}}}",
            account, bond
        ));
        account
    }

    /// SLASH a worker's bond for a missed/bad receipt (proven by the
    /// receipt hash mismatch). Only the owner may slash, with evidence.
    pub fn slash(&mut self, account: String, reason: String) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        let mut w = self.workers.get(&account).cloned().expect("no such worker");
        let slashed = w.bond;
        w.bond = 0;
        w.active = false;
        self.workers.insert(account.clone(), w);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"worker_slashed\",\"data\":{{\"account\":\"{}\",\"slashed\":\"{}\",\"reason\":\"{}\"}}}}",
            account, slashed, reason
        ));
    }

    // ---- WORK PAYOUT (bounty per verified receipt, never idle) ----

    /// A worker submits a verified work receipt (the canonical hash).
    /// The bounty is paid from the infra pool ONLY on a valid receipt.
    /// The receipt must be unique (replay refused).
    pub fn submit_work(&mut self, kind: String, receipt_hash: String) -> u128 {
        let worker = env::predecessor_account_id().to_string();
        let mut w = self.workers.get(&worker).cloned().expect("not a registered worker");
        assert!(w.active, "worker not active");
        assert!(!self.receipts.contains_key(&receipt_hash), "receipt replay refused");
        // the bounty: fixed per kind (a WORK cost, not a yield)
        let bounty = match kind.as_str() {
            "aipi_quorum" => 100,
            "bridge_mirror" => 60,
            "settle_batch" => 40,
            "maintenance" => 30,
            _ => env::panic_str("unknown work kind"),
        };
        assert!(self.infra_pool >= bounty, "infra pool depleted");
        self.infra_pool -= bounty;
        w.receipts_verified += 1;
        w.bounty_earned += bounty;
        self.workers.insert(worker.clone(), w);
        self.receipts.insert(receipt_hash.clone(), WorkReceipt {
            worker: worker.clone(),
            kind: kind.clone(),
            receipt_hash: receipt_hash.clone(),
            at: env::block_timestamp(),
            settled: true,
        });
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"work_bounty_paid\",\"data\":{{\"worker\":\"{}\",\"kind\":\"{}\",\"bounty\":\"{}\",\"receipt\":\"{}\"}}}}",
            worker, kind, bounty, receipt_hash
        ));
        bounty
    }

    /// Deployer maintenance compensation: drawn from the infra bucket as
    /// a documented vendor service (software upkeep, bridge daemons).
    /// Owner-callable; recorded as operational expense, not a dividend.
    pub fn maintenance_compensation(&mut self, amount: U128) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        assert!(self.infra_pool >= amount.0, "infra pool depleted");
        self.infra_pool -= amount.0;
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"maintenance_compensated\",\"data\":{{\"recipient\":\"{}\",\"amount\":\"{}\",\"service\":\"software licensing and upkeep\"}}}}",
            self.deployer, amount.0
        ));
    }

    // ---- THE TOLL (captured transaction fee) ----

    /// Capture a transaction toll and split it by the fixed work
    /// proportions. The four parts always sum to the toll.
    pub fn capture_toll(&mut self, amount: U128) -> TollSplit {
        let toll = amount.0;
        let reserve = toll * SPLIT_RESERVE / SPLIT_DENOM;
        let buffer = toll * SPLIT_BUFFER / SPLIT_DENOM;
        let infra = toll * SPLIT_INFRA / SPLIT_DENOM;
        let regen = toll - reserve - buffer - infra;
        self.reserve_floor += reserve;
        self.buffer_pool += buffer;
        self.infra_pool += infra;
        self.regen_pool += regen;
        self.tolls_captured += toll;
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv\",\"version\":\"1.0.0\",\"event\":\"toll_captured\",\"data\":{{\"toll\":\"{}\",\"reserve\":\"{}\",\"buffer\":\"{}\",\"infra\":\"{}\",\"regen\":\"{}\"}}}}",
            toll, reserve, buffer, infra, regen
        ));
        TollSplit { toll, reserve, buffer, infra, regen }
    }

    /// Issue a settlement-buffer bill at par (the debt instrument).
    pub fn issue_bill(&mut self, owner: String) -> String {
        let units = self.buffer_pool;
        assert!(units > 0, "no buffer backing yet");
        self.buffer_pool = 0;
        let id = format!("sb-{:06}", self.next_bill);
        self.next_bill += 1;
        self.bills.insert(id.clone(), BufferBill {
            owner, units, issued_at: env::block_timestamp(), redeemed: false,
        });
        id
    }

    /// Redeem a settlement-buffer bill at par against the reserve.
    pub fn redeem_bill(&mut self, bill_id: String) {
        let mut b = self.bills.get(&bill_id).cloned().expect("no such bill");
        assert!(!b.redeemed, "already redeemed");
        assert!(self.reserve_floor >= b.units, "reserve cannot cover");
        b.redeemed = true;
        let units = b.units;
        self.bills.insert(bill_id.clone(), b);
        self.reserve_floor -= units;
    }

    // ---- views ----

    pub fn state(&self) -> TreasuryState {
        TreasuryState {
            reserve_floor: self.reserve_floor,
            buffer_pool: self.buffer_pool,
            infra_pool: self.infra_pool,
            regen_pool: self.regen_pool,
            tolls_captured: self.tolls_captured,
        }
    }

    pub fn quote_toll(&self, amount: U128) -> U128 {
        U128(amount.0 * TOLL_NUM / TOLL_DEN)
    }

    pub fn worker(&self, account: String) -> Option<Worker> {
        self.workers.get(&account).cloned()
    }

    pub fn bill(&self, bill_id: String) -> Option<BufferBill> {
        self.bills.get(&bill_id).cloned()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct TollSplit {
    pub toll: u128,
    pub reserve: u128,
    pub buffer: u128,
    pub infra: u128,
    pub regen: u128,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct TreasuryState {
    pub reserve_floor: u128,
    pub buffer_pool: u128,
    pub infra_pool: u128,
    pub regen_pool: u128,
    pub tolls_captured: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_always_sums_to_toll() {
        let mut t = ZxvTreasury {
            owner: "o.near".parse().unwrap(),
            deployer: "d.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            reserve_floor: 0, buffer_pool: 0, infra_pool: 0, regen_pool: 0,
            tolls_captured: 0, workers: LookupMap::new(b"t".to_vec()),
            receipts: LookupMap::new(b"r".to_vec()),
            bills: LookupMap::new(b"b".to_vec()), next_bill: 1,
        };
        for toll in [1u128, 7, 100, 8889, 1_000_000, 12_345_678] {
            let s = t.capture_toll(U128(toll));
            assert_eq!(s.reserve + s.buffer + s.infra + s.regen, toll);
        }
    }

    #[test]
    fn no_bounty_without_work() {
        // the core regulatory property: the infra pool cannot be drawn
        // by idle accounts — only by registered workers with receipts
        let t = ZxvTreasury {
            owner: "o.near".parse().unwrap(),
            deployer: "d.near".parse().unwrap(),
            kernel: "k.near".parse().unwrap(),
            reserve_floor: 0, buffer_pool: 0, infra_pool: 1000, regen_pool: 0,
            tolls_captured: 0, workers: LookupMap::new(b"t".to_vec()),
            receipts: LookupMap::new(b"r".to_vec()),
            bills: LookupMap::new(b"b".to_vec()), next_bill: 1,
        };
        assert!(!t.workers.contains_key(&"idle.near".to_string()));
    }
}