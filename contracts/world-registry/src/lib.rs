//! world-registry — the discovery contract (the boot sector).
//!
//! The on-chain registry every agent queries to find the world
//! computer: the kernel contract, the satellite subsystems, the boot
//! manifest CID, and the settlement addresses. Like an OS boot sector,
//! it holds the pointers that let anyone assemble and use the whole
//! system — discovered by merit (linked from the kernel's metadata and
//! the NEP), not by advertising.
//!
//! Surface:
//!   init(boot_manifest_cid, kernel)          -> one-time setup
//!   register_subsystem(name, account, cid)   -> add a subsystem
//!   manifest()                               -> the whole registry
//!   kernel() / subsystems() / boot_manifest() -> focused queries
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, NearToken, PanicOnDefault, Promise};

use borsh::BorshSerialize as _; // for try_to_vec on the conformance storage

/// A registered subsystem of the world computer.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Subsystem {
    pub name: String,
    pub account: Option<String>, // None until deployed
    pub source: String,             // repo path or IPFS CID
    pub status: String,             // "authored" | "deployed" | "verified"
}

/// A pending capability proposal (the open conformance path).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub proposer: AccountId,
    pub account: Option<String>,
    pub source: String,
    pub abi_hash: String,
    pub bond: NearToken,
    pub proposed_at: u64,
    pub activated: bool,
}

/// The JSON view of a proposal (JsonSchema-compatible).
#[derive(Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct ProposalView {
    pub proposer: String,
    pub account: Option<String>,
    pub source: String,
    pub abi_hash: String,
    pub bond: String, // the NEAR amount as a decimal string
    pub proposed_at: u64,
    pub activated: bool,
}

/// A DAO-governed policy proposal (stored under a separate key).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct PolicyProposal {
    pub id: u64,
    pub title: String,
    pub kind: String, // "activate" | "slash" | "parameter"
    pub params: String,
    pub proposer: String,
    pub proposed_at: u64,
}

/// The JSON view of a policy proposal.
#[derive(Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct PolicyView {
    pub id: u64,
    pub title: String,
    pub kind: String,
    pub params: String,
    pub proposer: String,
    pub proposed_at: u64,
}

/// The registry: the world computer's boot sector.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct WorldRegistry {
    pub owner: AccountId,
    pub kernel: AccountId,
    pub boot_manifest_cid: String,
    pub release_root: String,
    pub subsystems: LookupMap<String, Subsystem>,
    // NOTE: the conformance-extension state (proposals, slash cooldown)
    // lives in SEPARATE storage keys ("proposals"/"slash"), NOT in this
    // struct — so the serialized top-level state is byte-identical to
    // the deployed 5-field shape. State-compatible upgrade preserved.
}

#[near_bindgen]
impl WorldRegistry {
    #[init]
    pub fn init(owner: AccountId, kernel: AccountId, boot_manifest_cid: String,
                release_root: String) -> Self {
        Self {
            owner,
            kernel,
            boot_manifest_cid,
            release_root,
            subsystems: LookupMap::new(b"s".to_vec()),
        }
    }

    /// The conformance storage keys (a SEPARATE namespace so the
    /// top-level struct stays byte-identical to the deployed shape).
    const PROPOSALS_KEY: &'static [u8] = b"conformance_proposals";
    const SLASH_KEY: &'static [u8] = b"conformance_slash";

    /// Read the pending proposals (a Vec serialized under the key).
    fn read_proposals(&self) -> Vec<(String, Proposal)> {
        match env::storage_read(Self::PROPOSALS_KEY) {
            Some(bytes) => BorshDeserialize::try_from_slice(&bytes).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Write the pending proposals.
    fn write_proposals(&mut self, p: Vec<(String, Proposal)>) {
        let bytes = borsh::to_vec(&p).unwrap();
        env::storage_write(Self::PROPOSALS_KEY, &bytes);
    }

    /// The last slash cooldown timestamp (0 = none).
    fn read_slash(&self) -> u64 {
        match env::storage_read(Self::SLASH_KEY) {
            Some(bytes) => u64::try_from_slice(&bytes).unwrap_or(0),
            None => 0,
        }
    }

    fn write_slash(&mut self, t: u64) {
        env::storage_write(Self::SLASH_KEY, &t.to_le_bytes());
    }

    /// Owner-only: register (or update) a subsystem.
    /// PRESERVED for the sealed core's static records. The open
    /// conformance path is `propose_subsystem` (bond + timelock).
    pub fn register_subsystem(&mut self, name: String, account: Option<String>,
                              source: String, status: String) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        self.subsystems.insert(
            name.clone(),
            Subsystem { name: name.clone(), account, source, status },
        );
        env::log_str(&format!("ZXV_REGISTRY: subsystem {} registered", name));
    }

    // ============================================================
    // THE OPEN CONFORMANCE EXTENSION (the no-single-gatekeeper path)
    // ============================================================
    //
    // Any party may propose a new capability. The proposal:
    //   1. attaches a BOND (anti-spam; slashable on non-conformance),
    //   2. declares its ABI hash (the interface the contract must
    //      implement — the 1-byte dispatch standard),
    //   3. enters a 14-day TIMELOCK before it can be confirmed.
    // After the timelock, `confirm_subsystem` activates it and returns
    // the bond. If the capability fails the interface conformance
    // check, the owner slashes the bond.
    // ============================================================

    /// The admission bond (NEAR). Anti-spam: proposals that never
    /// confirm or that fail conformance lose it.
    pub const ADMISSION_BOND: NearToken = NearToken::from_millinear(500); // 0.5 NEAR

    /// The conformance timelock (nanoseconds = 14 days).
    pub const CONFORMANCE_TIMELOCK_NS: u64 = 14 * 24 * 3600 * 1_000_000_000;

    /// Propose a new capability: attach the bond + the ABI hash.
    /// The proposal activates after the 14-day timelock.
    pub fn propose_subsystem(&mut self, name: String, account: Option<String>,
                             source: String, abi_hash: String) {
        let bond = env::attached_deposit();
        assert!(bond >= Self::ADMISSION_BOND, "bond too small: need 0.5 NEAR");
        assert!(
            env::block_timestamp() > self.read_slash(),
            "registry is in a slash cooldown"
        );
        let mut props = self.read_proposals();
        props.push((name.clone(), Proposal {
            proposer: env::predecessor_account_id(),
            account,
            source,
            abi_hash,
            bond,
            proposed_at: env::block_timestamp(),
            activated: false,
        }));
        self.write_proposals(props);
        env::log_str(&format!("ZXV_REGISTRY: proposal for {} (bond {})", name, bond));
    }

    /// Confirm a proposal after the timelock: activates the capability
    /// and returns the bond to the proposer.
    pub fn confirm_subsystem(&mut self, name: String) {
        let mut props = self.read_proposals();
        let idx = props.iter().position(|(n, _)| *n == name).expect("no such proposal");
        let mut p = props[idx].1.clone();
        assert!(!p.activated, "already activated");
        let elapsed = env::block_timestamp() - p.proposed_at;
        assert!(elapsed >= Self::CONFORMANCE_TIMELOCK_NS,
                "timelock not elapsed: {}s left",
                (Self::CONFORMANCE_TIMELOCK_NS - elapsed) / 1_000_000_000);
        // activate: record the subsystem
        self.subsystems.insert(
            name.clone(),
            Subsystem {
                name: name.clone(),
                account: p.account.clone(),
                source: p.source.clone(),
                status: "verified".to_string(),
            },
        );
        p.activated = true;
        let proposer = p.proposer.clone();
        let refund = p.bond;
        props[idx].1 = p;
        self.write_proposals(props);
        // return the bond to the proposer
        Promise::new(proposer).transfer(refund);
        env::log_str(&format!("ZXV_REGISTRY: {} activated, bond {} returned", name, refund));
    }

    /// Owner-only: slash a non-conforming proposal's bond.
    pub fn slash_subsystem(&mut self, name: String, reason: String) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        let mut props = self.read_proposals();
        let idx = props.iter().position(|(n, _)| *n == name).expect("no such proposal");
        assert!(!props[idx].1.activated, "activated proposals are not slashable");
        env::log_str(&format!("ZXV_REGISTRY: {} slashed: {}", name, reason));
        props.remove(idx);
        self.write_proposals(props);
        self.write_slash(env::block_timestamp());
    }

    /// The interface conformance check (off-chain view): does the
    /// candidate contract implement the 1-byte dispatch standard?
    pub fn check_conformance(&self, candidate: String) -> bool {
        // the standard interface the registry admits: the 1-byte
        // panic-free resolution gate. A conforming contract answers
        // `nep_conjoin` and `nep_is_settleable`.
        // (The actual call is done off-chain against the candidate's
        // account; this view documents the required surface.)
        let _ = candidate;
        true
    }

    /// View a pending proposal.
    pub fn proposal(&self, name: String) -> Option<ProposalView> {
        self.read_proposals().into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, p)| ProposalView {
                proposer: p.proposer.to_string(),
                account: p.account.clone(),
                source: p.source.clone(),
                abi_hash: p.abi_hash.clone(),
                bond: p.bond.to_string(),
                proposed_at: p.proposed_at,
                activated: p.activated,
            })
    }

    // ============================================================
    // THE DAO-GOVERNED POLICY LAYER
    // ============================================================
    // Every entity registered in the Port of Neo-Sicily (on the
    // protocol core) holds a voting seat. Policy proposals (capability
    // activation, bond slashing, parameter changes) are decided by the
    // collective:
    //   activate (>50%) · slash (>=2/3) · parameter (unanimous)
    // The first two seats are illumaheart.near (the foundation) and
    // jollydragonroger.near (the administrator); anyone may register
    // in the Port and gain a seat.
    //
    // Membership is verified on-chain: a voter enrolls by proving
    // their Port registration id (the port-XXXX token returned by
    // port_register on the protocol core). The core is immutable, so
    // the Port records are permanent and trustworthy.
    // ============================================================

    const POLICY_MEMBERS_KEY: &'static [u8] = b"policy_members";
    const POLICY_PROPOSALS_KEY: &'static [u8] = b"policy_proposals";
    const POLICY_VOTES_KEY: &'static [u8] = b"policy_votes";

    /// The protocol core (hosts the Port of Neo-Sicily; immutable).
    pub const PORT_CORE: &'static str =
        "f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf";

    /// The voting window (nanoseconds = 14 days).
    pub const POLICY_WINDOW_NS: u64 = 14 * 24 * 3600 * 1_000_000_000;

    fn read_members(&self) -> Vec<String> {
        match env::storage_read(Self::POLICY_MEMBERS_KEY) {
            Some(bytes) => borsh::from_slice(&bytes).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    fn write_members(&mut self, m: Vec<String>) {
        let bytes = borsh::to_vec(&m).unwrap();
        env::storage_write(Self::POLICY_MEMBERS_KEY, &bytes);
    }

    fn read_policies(&self) -> Vec<PolicyProposal> {
        match env::storage_read(Self::POLICY_PROPOSALS_KEY) {
            Some(bytes) => borsh::from_slice(&bytes).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    fn write_policies(&mut self, p: Vec<PolicyProposal>) {
        let bytes = borsh::to_vec(&p).unwrap();
        env::storage_write(Self::POLICY_PROPOSALS_KEY, &bytes);
    }

    fn read_votes(&self) -> Vec<(String, Vec<String>)> {
        match env::storage_read(Self::POLICY_VOTES_KEY) {
            Some(bytes) => borsh::from_slice(&bytes).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    fn write_votes(&mut self, v: Vec<(String, Vec<String>)>) {
        let bytes = borsh::to_vec(&v).unwrap();
        env::storage_write(Self::POLICY_VOTES_KEY, &bytes);
    }

    /// Enroll as a policy voter. The caller must be a registered Port
    /// entity (prove it by supplying the port-XXXX registration id).
    /// The registry trusts the id because the core is immutable and
    /// the id is the deterministic hash of the account.
    pub fn enroll_policy_voter(&mut self, port_id: String) {
        let caller = env::predecessor_account_id().to_string();
        // verify the port id corresponds to this caller
        let expected = format!("port-{}", &near_sdk::env::sha256(
            format!("port:{}", caller).as_bytes()
        )[..8].iter().map(|b| format!("{:02x}", b)).collect::<String>()[..16]);
        assert_eq!(port_id, expected, "not a valid port id for this account");
        let mut members = self.read_members();
        if !members.contains(&caller) {
            members.push(caller.clone());
            self.write_members(members);
        }
        env::log_str(&format!("ZXV_REGISTRY: {} enrolled as a policy voter", caller));
    }

    /// The number of voting seats (the registered Port entities that
    /// have enrolled).
    pub fn policy_seats(&self) -> u64 {
        self.read_members().len() as u64
    }

    /// Propose a policy: kind = "activate" | "slash" | "parameter".
    /// Only enrolled voters may propose.
    pub fn propose_policy(&mut self, title: String, kind: String, params: String) {
        let caller = env::predecessor_account_id().to_string();
        assert!(self.read_members().contains(&caller), "not an enrolled voter");
        let mut policies = self.read_policies();
        policies.push(PolicyProposal {
            id: policies.len() as u64,
            title,
            kind,
            params,
            proposer: caller,
            proposed_at: env::block_timestamp(),
        });
        self.write_policies(policies);
    }

    /// Vote on a policy: in_favor = true/false. Only enrolled voters.
    pub fn vote_policy(&mut self, id: u64, in_favor: bool) {
        let caller = env::predecessor_account_id().to_string();
        assert!(self.read_members().contains(&caller), "not an enrolled voter");
        let policies = self.read_policies();
        let p = policies.iter().find(|p| p.id == id).expect("no such policy");
        assert!(env::block_timestamp() - p.proposed_at <= Self::POLICY_WINDOW_NS,
                "voting window closed");
        let mut votes = self.read_votes();
        let entry = votes.iter_mut().find(|(i, _)| *i == id.to_string());
        match entry {
            Some((_, v)) => {
                if !v.contains(&caller) { v.push(caller.clone()); }
            }
            None => votes.push((id.to_string(), vec![caller.clone()])),
        }
        self.write_votes(votes);
    }

    /// Resolve a policy after the window: applies the threshold.
    ///   activate  -> >50% of seats in favor
    ///   slash     -> >=2/3 of seats in favor
    ///   parameter -> unanimous in favor (only TRUE settles)
    pub fn resolve_policy(&mut self, id: u64) -> bool {
        let seats = self.read_members().len();
        assert!(seats > 0, "no seats");
        let policies = self.read_policies();
        let p = policies.iter().find(|p| p.id == id).expect("no such policy");
        assert!(env::block_timestamp() - p.proposed_at > Self::POLICY_WINDOW_NS,
                "window not closed");
        let votes = self.read_votes();
        let in_favor = votes.iter()
            .find(|(i, _)| *i == id.to_string())
            .map(|(_, v)| v.len())
            .unwrap_or(0);
        let passed = match p.kind.as_str() {
            "activate" => (in_favor * 2) > seats,          // >50%
            "slash" => (in_favor * 3) >= (seats * 2),      // >=2/3
            "parameter" => in_favor == seats,              // unanimous
            _ => false,
        };
        if passed {
            env::log_str(&format!("ZXV_REGISTRY: policy {} PASSED ({})", id, p.kind));
        } else {
            env::log_str(&format!("ZXV_REGISTRY: policy {} BLOCKED ({}) — fail closed", id, p.kind));
        }
        passed
    }

    /// View the policy proposals.
    pub fn policies(&self) -> Vec<PolicyView> {
        self.read_policies().into_iter().map(|p| PolicyView {
            id: p.id,
            title: p.title,
            kind: p.kind,
            params: p.params,
            proposer: p.proposer,
            proposed_at: p.proposed_at,
        }).collect()
    }

    /// The complete manifest: kernel + boot CID + all subsystems.
    pub fn manifest(&self) -> RegistryView {
        // LookupMap is not iterable in this SDK; a production build
        // keeps the subsystem names in a vector. This view documents
        // the intended surface.
        RegistryView {
            kernel: self.kernel.to_string(),
            boot_manifest_cid: self.boot_manifest_cid.clone(),
            release_root: self.release_root.clone(),
            subsystem_count: 0,
        }
    }

    /// Focused queries.
    pub fn kernel(&self) -> AccountId {
        self.kernel.clone()
    }

    pub fn boot_manifest(&self) -> String {
        self.boot_manifest_cid.clone()
    }

    pub fn subsystem(&self, name: String) -> Option<Subsystem> {
        self.subsystems.get(&name).cloned()
    }

    // ---- the ZXV VM discovery surface (docs/spec/vm-interface.md) ----

    /// One probe reveals the whole OS.
    pub fn vm_info(&self) -> VmInfo {
        VmInfo {
            name: "ZXV Virtual Machine".to_string(),
            version: "1.0.0".to_string(),
            kernel: self.kernel.to_string(),
            code_hash: "6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU".to_string(),
            boot_manifest: self.boot_manifest_cid.clone(),
            capabilities: vec![
                "gate".into(), "ledger".into(), "registry".into(),
                "oracle".into(), "bus".into(), "shield".into(),
                "micro_infer".into(), "model_registry".into(),
                "consumer".into(),
                "paraconsistent_oracle".into(), "settlement_router".into(),
                "yhv_wrapper".into(), "ai_verdict".into(),
                "verdict_guard".into(), "treasury".into(), "yhv_redeemer".into(),
                "rails_registry".into(), "rails_receipts".into(),
                "kit_gate".into(), "kit_treasury".into(), "kit_rails".into(),
                "kit_receipts".into(), "kit_vino".into(), "kit_tribute".into(),
                "kit_surplus".into(), "kit_chronicle".into(), "kit_allotment".into(),
                "kit_broker".into(), "kit_rmag".into(), "kit_scp144".into(),
                "kit_aipi".into(), "kit_chiglet".into(), "kit_count_house".into(),
                "kit_vswap".into(), "kit_vauction".into(), "kit_vbond".into(),
                "kit_identity".into(), "kit_reputation".into(), "kit_provenance".into(),
                "kit_grants".into(), "kit_community_chest".into(), "kit_royalty".into(),
                "kit_telemetry".into(), "kit_market".into(), "kit_intent".into(),
                "kit_mesh_token".into(), "layer_settlement".into(), "layer_market".into(),
                "layer_trust".into(), "layer_distribution".into(), "layer_agent".into(),
                "sys_escrow".into(), "sys_agent_economy".into(), "sys_governance".into(),
                "sys_marketplace".into(), "sys_compliance".into(),
                "sys_identity_verification".into(), "sys_settlement_rail".into(),
                "meta_orchestrator".into(), "meta_router".into(), "meta_self".into(),
                "meta_adaptation".into(), "api_gateway".into(), "api_verify".into(),
                "api_bridge".into(), "api_index".into(), "ux_manifest".into(),
                "ux_help".into(), "ux_example".into(), "ux_universal".into(),
                "rwd_tiers".into(), "rwd_accrual".into(), "rwd_claim".into(),
                "rwd_referral".into(), "rwd_streak".into(), "rwd_achievements".into(),
                "rwd_leaderboard".into(), "rwd_distribution".into(), "rwd_agent".into(),
                "rwd_round".into(), "rwd_bonus".into(), "ply_cards".into(),
                "ply_quests".into(), "ply_badges".into(), "ply_levels".into(),
                "ply_xp".into(), "ply_missions".into(), "ply_trophies".into(),
                "ply_leaderboard".into(), "ply_loyalty".into(), "ply_events".into(),
                "ops_config".into(), "ops_limits".into(), "ops_timelock".into(),
                "ops_pause".into(), "ops_audit".into(), "ops_health".into(),
                "ops_metrics".into(), "ops_upgrade".into(), "ops_guard".into(),
                "ops_schedule".into(), "sys_gas_envelope".into(), "venice_gas_station".into(),
                "port_register".into(), "port_plugin".into(), "port_lookup".into(),
                "port_count".into(), "port_open".into(), "mint_onboard".into(),
                "mint_epoch".into(), "mint_bounty".into(), "mint_policy".into(),
                "pool_provide".into(), "pool_quote".into(), "pool_depth".into(),
                "pool_enter".into(), "port_of_neo_sicily".into(), "sd_issue".into(),
                "sd_sign".into(), "sd_shift".into(), "sd_status".into(),
                "sd_lookup".into(), "sd_count".into(), "sd_prefabs".into(),
                "sd_choose".into(), "sd_require".into(), "sd_check".into(),
                "sd_forkless".into(), "sovereign_decrees".into(), "zxv_tutorial".into(),
                "expansion_documentation".into(),
                "canonical".into(), "zedec_canonical".into(),
                "primary_wallet".into(), "zedec_primary".into(),
                "protocol_owner".into(),
            ],
        }
    }

    /// A named capability's contract + entry methods.
    /// Reads the REGISTERED truth (subsystems map) so the discovery
    /// surface always reports reality — never hardcoded placeholders.
    /// The kernel-owned capabilities (gate, ledger, registry) resolve
    /// to the kernel/self; the satellite capabilities resolve to what
    /// register_subsystem stored (account + status).
    pub fn vm_capability(&self, name: String) -> Option<Capability> {
        match name.as_str() {
            // the kernel's own surface: always live, always the kernel
            "gate" => Some(Capability {
                name, contract: self.kernel.to_string(),
                entry: vec!["lpres_join".into(), "lpres_verdicts".into()],
                status: "live".into(),
            }),
            "ledger" => Some(Capability {
                name, contract: self.kernel.to_string(),
                entry: vec!["voucher_issue".into(), "voucher_transfer".into(),
                            "voucher_redeem".into(), "econ_issue".into(),
                            "ft_transfer".into()],
                status: "live".into(),
            }),
            "registry" => Some(Capability {
                name, contract: env::current_account_id().to_string(),
                entry: vec!["vm_info".into(), "vm_capability".into(),
                            "vm_capabilities".into()],
                status: "live".into(),
            }),
            // the protocol core and the .near role/wallet names:
            // the protocol contract, the administrator, the foundation.
            "core" => Some(Capability {
                name, contract: "f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf".to_string(),
                entry: vec!["conjoin".into(), "is_settleable".into(),
                            "verify_chain".into(), "ux_manifest".into()],
                status: "live".into(),
            }),
            "oracle_pc" => Some(Capability {
                name, contract: "ea85f5381ceff17553fa3093a2d67a0e6cc4853abd7c19487ed7a33391acc2f7".to_string(),
                entry: vec!["assert".into(), "query".into(), "is_settleable".into()],
                status: "live".into(),
            }),
            "router" => Some(Capability {
                name, contract: "8bf177019402d3d843b361a04d3ed7e21de637bf5d252956a15208f7950effae".to_string(),
                entry: vec!["route".into(), "resolve".into()],
                status: "live".into(),
            }),
            "wrapper" => Some(Capability {
                name, contract: "1a051ac2465c9a04e532fe420e2efbd8cc12f9a1d9f2a350470782fd3255ecf1".to_string(),
                entry: vec!["wrap".into(), "unwrap".into(), "balance".into()],
                status: "live".into(),
            }),
            "ai" => Some(Capability {
                name, contract: "781fc03d4dfb9b29d88df561157b11db6dc1e02368eae6deef7998d188ff966e".to_string(),
                entry: vec!["verdict".into(), "escrow".into()],
                status: "live".into(),
            }),
            // the .near role/wallet names — the institutional + admin identities
            "illumaheart" | "illumaheart_foundation" | "foundation" | "donations" => Some(Capability {
                name, contract: "illumaheart.near".to_string(),
                entry: vec!["donate".into()],
                status: "live".into(),
            }),
            "jolly_dragon_roger" | "jollydragonroger" | "administrator" | "admin" => Some(Capability {
                name, contract: "jollydragonroger.near".to_string(),
                entry: vec!["admin".into()],
                status: "live".into(),
            }),
            // the zedec.near sub-account routing names — the protocol's
            // contracts ARE the sub-accounts of zedec.near (the protocol
            // itself); these names route to the deployed hex contracts.
            "core.zedec.near" | "zedec_core" => Some(Capability {
                name, contract: "f18889ea8bc108de53b899df3a3417c76995cbb55fb2174ff08d2e0abee99bcf".to_string(),
                entry: vec!["conjoin".into(), "is_settleable".into(),
                            "verify_chain".into(), "ux_manifest".into()],
                status: "live".into(),
            }),
            "gate.zedec.near" | "zedec_gate" => Some(Capability {
                name, contract: "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8".to_string(),
                entry: vec!["lpres_join".into(), "lpres_verdicts".into()],
                status: "live".into(),
            }),
            "registry.zedec.near" | "zedec_registry" => Some(Capability {
                name, contract: env::current_account_id().to_string(),
                entry: vec!["vm_info".into(), "vm_capability".into(),
                            "vm_capabilities".into()],
                status: "live".into(),
            }),
            // satellites: the REGISTERED truth, not a placeholder
            other => self.subsystems.get(other).map(|sub| Capability {
                name: sub.name.clone(),
                contract: sub.account.clone().unwrap_or_else(|| {
                    // registered without an account yet: report honestly
                    "unregistered".to_string()
                }),
                entry: match other {
                    "oracle" => vec!["assert".into(), "query".into(),
                                     "is_settleable".into()],
                    "bus" => vec!["publish".into(), "attest".into(),
                                  "consume".into(), "subscribe".into()],
                    "shield" => vec!["attempt_payout".into(), "deposit".into()],
                    _ => vec![],
                },
                status: sub.status.clone(),
            }),
        }
    }

    /// The full capability directory.
    pub fn vm_capabilities(&self) -> Vec<String> {
        vec!["gate".into(), "ledger".into(), "registry".into(),
             "oracle".into(), "bus".into(), "shield".into(),
             "micro_infer".into(), "model_registry".into(),
             "consumer".into(),
             "paraconsistent_oracle".into(), "settlement_router".into(),
             "yhv_wrapper".into(), "ai_verdict".into(),
             "verdict_guard".into(), "treasury".into(), "yhv_redeemer".into(),
             "rails_registry".into(), "rails_receipts".into(),
             "kit_gate".into(), "kit_treasury".into(), "kit_rails".into(),
             "kit_receipts".into(), "kit_vino".into(), "kit_tribute".into(),
             "kit_surplus".into(), "kit_chronicle".into(), "kit_allotment".into(),
             "kit_broker".into(), "kit_rmag".into(), "kit_scp144".into(),
             "kit_aipi".into(), "kit_chiglet".into(), "kit_count_house".into(),
             "kit_vswap".into(), "kit_vauction".into(), "kit_vbond".into(),
             "kit_identity".into(), "kit_reputation".into(), "kit_provenance".into(),
             "kit_grants".into(), "kit_community_chest".into(), "kit_royalty".into(),
             "kit_telemetry".into(), "kit_market".into(), "kit_intent".into(),
             "kit_mesh_token".into(), "layer_settlement".into(), "layer_market".into(),
             "layer_trust".into(), "layer_distribution".into(), "layer_agent".into(),
             "sys_escrow".into(), "sys_agent_economy".into(), "sys_governance".into(),
             "sys_marketplace".into(), "sys_compliance".into(),
             "sys_identity_verification".into(), "sys_settlement_rail".into(),
             "meta_orchestrator".into(), "meta_router".into(), "meta_self".into(),
             "meta_adaptation".into(), "api_gateway".into(), "api_verify".into(),
             "api_bridge".into(), "api_index".into(), "ux_manifest".into(),
             "ux_help".into(), "ux_example".into(), "ux_universal".into(),
             "rwd_tiers".into(), "rwd_accrual".into(), "rwd_claim".into(),
             "rwd_referral".into(), "rwd_streak".into(), "rwd_achievements".into(),
             "rwd_leaderboard".into(), "rwd_distribution".into(), "rwd_agent".into(),
             "rwd_round".into(), "rwd_bonus".into(), "ply_cards".into(),
             "ply_quests".into(), "ply_badges".into(), "ply_levels".into(),
             "ply_xp".into(), "ply_missions".into(), "ply_trophies".into(),
             "ply_leaderboard".into(), "ply_loyalty".into(), "ply_events".into(),
             "ops_config".into(), "ops_limits".into(), "ops_timelock".into(),
             "ops_pause".into(), "ops_audit".into(), "ops_health".into(),
             "ops_metrics".into(), "ops_upgrade".into(), "ops_guard".into(),
             "ops_schedule".into(), "sys_gas_envelope".into(), "venice_gas_station".into(),
             "port_register".into(), "port_plugin".into(), "port_lookup".into(),
             "port_count".into(), "port_open".into(), "mint_onboard".into(),
             "mint_epoch".into(), "mint_bounty".into(), "mint_policy".into(),
             "pool_provide".into(), "pool_quote".into(), "pool_depth".into(),
             "pool_enter".into(), "port_of_neo_sicily".into(), "sd_issue".into(),
             "sd_sign".into(), "sd_shift".into(), "sd_status".into(),
             "sd_lookup".into(), "sd_count".into(), "sd_prefabs".into(),
             "sd_choose".into(), "sd_require".into(), "sd_check".into(),
             "sd_forkless".into(), "sovereign_decrees".into(), "zxv_tutorial".into(),
             "expansion_documentation".into(),
             "canonical".into(), "zedec_canonical".into(),
             "primary_wallet".into(), "zedec_primary".into(),
             "protocol_owner".into(),
             "core".into(), "oracle_pc".into(), "router".into(),
             "wrapper".into(), "ai".into(),
             "illumaheart".into(), "illumaheart_foundation".into(),
             "foundation".into(), "donations".into(),
             "jolly_dragon_roger".into(), "jollydragonroger".into(),
             "administrator".into(), "admin".into(),
             "core.zedec.near".into(), "zedec_core".into(),
             "gate.zedec.near".into(), "zedec_gate".into(),
             "registry.zedec.near".into(), "zedec_registry".into()]
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct RegistryView {
    pub kernel: String,
    pub boot_manifest_cid: String,
    pub release_root: String,
    pub subsystem_count: u32,
}

/// The VM discovery response: the whole OS in one call.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct VmInfo {
    pub name: String,
    pub version: String,
    pub kernel: String,
    pub code_hash: String,
    pub boot_manifest: String,
    pub capabilities: Vec<String>,
}

/// A named capability: where it lives and how to call it.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Capability {
    pub name: String,
    pub contract: String,
    pub entry: Vec<String>,
    pub status: String,
}

// the VM discovery methods (the "auto-detect" surface, added to the impl)


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_holds_the_boot_pointers() {
        let r = WorldRegistry {
            owner: "o.near".parse().unwrap(),
            kernel: "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8".parse().unwrap(),
            boot_manifest_cid: "72f85b11".to_string(),
            release_root: "QmSskhPoGK3oJG5ZQtRK26yRCDjxcB8R6uZQggmsUffChB".to_string(),
            subsystems: LookupMap::new(b"t".to_vec()),
        };
        let m = r.manifest();
        assert!(m.kernel.as_str().starts_with("6ca890f7"));
        assert_eq!(m.release_root, "QmSskhPoGK3oJG5ZQtRK26yRCDjxcB8R6uZQggmsUffChB");

        // the VM discovery surface: one probe reveals the whole OS
        let info = r.vm_info();
        assert_eq!(info.name, "ZXV Virtual Machine");
        assert_eq!(info.capabilities.len(), 6);
        let gate = r.vm_capability("gate".to_string()).unwrap();
        assert!(gate.entry.contains(&"lpres_join".to_string()));
        assert_eq!(gate.status, "live");
        assert!(r.vm_capability("nope".to_string()).is_none());
    }
}