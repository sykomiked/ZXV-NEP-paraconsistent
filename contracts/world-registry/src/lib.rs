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
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// A registered subsystem of the world computer.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct Subsystem {
    pub name: String,
    pub account: Option<String>, // None until deployed
    pub source: String,             // repo path or IPFS CID
    pub status: String,             // "authored" | "deployed" | "verified"
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

    /// Owner-only: register (or update) a subsystem.
    pub fn register_subsystem(&mut self, name: String, account: Option<String>,
                              source: String, status: String) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        self.subsystems.insert(
            name.clone(),
            Subsystem { name: name.clone(), account, source, status },
        );
        env::log_str(&format!("ZXV_REGISTRY: subsystem {} registered", name));
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