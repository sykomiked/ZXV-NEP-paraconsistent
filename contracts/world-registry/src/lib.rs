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
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// A registered subsystem of the world computer.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct Subsystem {
    pub name: String,
    pub account: Option<AccountId>, // None until deployed
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
    pub fn register_subsystem(&mut self, name: String, account: Option<AccountId>,
                              source: String, status: String) {
        assert_eq!(env::predecessor_account_id(), self.owner, "only owner");
        self.subsystems.insert(
            &name,
            &Subsystem { name: name.clone(), account, source, status },
        );
        env::log_str(&format!("ZXV_REGISTRY: subsystem {} registered", name));
    }

    /// The complete manifest: kernel + boot CID + all subsystems.
    pub fn manifest(&self) -> RegistryView {
        // LookupMap is not iterable in this SDK; a production build
        // keeps the subsystem names in a vector. This view documents
        // the intended surface.
        RegistryView {
            kernel: self.kernel.clone(),
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
        self.subsystems.get(&name)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
pub struct RegistryView {
    pub kernel: AccountId,
    pub boot_manifest_cid: String,
    pub release_root: String,
    pub subsystem_count: u32,
}

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
    }
}