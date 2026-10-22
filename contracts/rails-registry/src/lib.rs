//! rails-registry — the ZXV rail card networks (Dragon, Phoenix, Thunderbird).
//!
//! The identity layer of the rail system (rails spec D1). Cards are
//! 20-digit Luhn-valid rail numbers OUTSIDE ISO/IEC 7812 — ZXV's own
//! identifiers, never payment-card numbers (no IIN, no PAN semantics).
//!
//!   Dragon      prefix 8809, inner check ISO/IEC 7064 MOD 11,10
//!   Phoenix     prefix 8829, inner check Damm
//!   Thunderbird prefix 8849, inner check Verhoeff
//!
//! The check-digit algorithms are ported byte-for-byte from the kernel's
//! cn_check.c so the contract agrees with the C reference, the Xahau
//! hooks, and the SDKs. A rail number is not a secret (safe to print);
//! spending always needs the holder's signature. Cards are
//! NON-TRANSFERABLE (NEP-171 read surface; nft_transfer always fails).
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

/// The three rail networks.
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
#[borsh(use_discriminant = true)]
pub enum RailNetwork {
    Dragon = 0,
    Phoenix = 1,
    Thunderbird = 2,
}

impl RailNetwork {
    const fn prefix(self) -> &'static str {
        match self {
            RailNetwork::Dragon => "8809",
            RailNetwork::Phoenix => "8829",
            RailNetwork::Thunderbird => "8849",
        }
    }
}

/// A card: the on-chain identity record.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, schemars::JsonSchema)]
pub struct RailCard {
    pub number: String,       // the 20-digit rail number
    pub network: RailNetwork,
    pub holder: String,       // the "name on the card" = the public key/account
    pub kind: String,         // "person" | "agent"
    pub parent: Option<String>, // agent cards: the funding card
    pub generation: u32,
    pub state: String,        // active | frozen | replaced | closed
}

// ---- the check-digit algorithms (ported from cn_check.c) ----

fn luhn_sum(digits: &[u8], double_last: bool) -> u32 {
    // the kernel's luhn_sum: iterate from the RIGHT; when double_last is
    // set, the rightmost digit is doubled first
    let mut sum = 0u32;
    let mut dbl = double_last;
    for i in (0..digits.len()).rev() {
        let mut v = (digits[i] - b'0') as u32;
        if dbl {
            v *= 2;
            if v > 9 { v -= 9; }
        }
        sum += v;
        dbl = !dbl;
    }
    sum % 10
}

fn luhn_valid(digits: &[u8]) -> bool {
    if digits.len() < 2 { return false; }
    luhn_sum(digits, false) == 0
}

fn mod1110_run(digits: &[u8]) -> u32 {
    let mut p = 10u32;
    for &d in digits {
        let mut s = (p + (d - b'0') as u32) % 10;
        if s == 0 { s = 10; }
        p = (2 * s) % 11;
    }
    p
}

fn mod1110_valid(digits: &[u8]) -> bool {
    if digits.len() < 2 { return false; }
    let p = mod1110_run(&digits[..digits.len() - 1]);
    (p + (digits[digits.len() - 1] - b'0') as u32) % 10 == 1
}

// the Damm table, exactly as in cn_check.c
const DAMM: [[u8; 10]; 10] = [
    [0, 3, 1, 7, 5, 9, 8, 6, 4, 2], [7, 0, 9, 2, 1, 5, 4, 8, 6, 3],
    [4, 2, 0, 6, 8, 7, 1, 3, 5, 9], [1, 7, 5, 0, 9, 8, 3, 4, 2, 6],
    [6, 1, 2, 3, 0, 4, 5, 9, 7, 8], [3, 6, 7, 4, 2, 0, 9, 5, 8, 1],
    [5, 8, 6, 9, 7, 2, 0, 1, 3, 4], [8, 9, 4, 5, 3, 6, 2, 0, 1, 7],
    [9, 4, 3, 8, 6, 1, 7, 2, 0, 5], [2, 5, 8, 1, 4, 3, 6, 7, 9, 0],
];

fn damm_valid(digits: &[u8]) -> bool {
    if digits.len() < 2 { return false; }
    let mut interim = 0usize;
    for &d in digits {
        interim = DAMM[interim][(d - b'0') as usize] as usize;
    }
    interim == 0
}

// the Verhoeff multiplication, permutation and inverse tables
const V_D: [[u8; 10]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], [1, 2, 3, 4, 0, 6, 7, 8, 9, 5],
    [2, 3, 4, 0, 1, 7, 8, 9, 5, 6], [3, 4, 0, 1, 2, 8, 9, 5, 6, 7],
    [4, 0, 1, 2, 3, 9, 5, 6, 7, 8], [5, 9, 8, 7, 6, 0, 4, 3, 2, 1],
    [6, 5, 9, 8, 7, 1, 0, 4, 3, 2], [7, 6, 5, 9, 8, 2, 1, 0, 4, 3],
    [8, 7, 6, 5, 9, 3, 2, 1, 0, 4], [9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];
const V_P: [[u8; 10]; 8] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], [1, 5, 7, 6, 2, 8, 3, 0, 9, 4],
    [5, 8, 0, 3, 7, 9, 6, 1, 4, 2], [8, 9, 1, 6, 0, 4, 3, 5, 2, 7],
    [9, 4, 5, 3, 1, 2, 6, 8, 7, 0], [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
    [2, 7, 9, 3, 8, 0, 6, 4, 1, 5], [7, 0, 4, 6, 9, 1, 3, 2, 5, 8],
];
const V_INV: [u8; 10] = [0, 4, 3, 2, 1, 5, 6, 7, 8, 9];

fn verhoeff_run(digits: &[u8], mode: usize) -> usize {
    // mode 0: validity (starts c at 0, checks V_INV[c] == 0)
    // mode 1: digit (starts c at 1, returns V_INV[c]) — the kernel's
    //         cn_verhoeff_digit uses verhoeff_run(digits, len, 1)
    let mut c = mode;
    for (i, &d) in digits.iter().rev().enumerate() {
        c = V_D[c][V_P[i % 8][(d - b'0') as usize] as usize] as usize;
    }
    c
}

fn verhoeff_valid(digits: &[u8]) -> bool {
    if digits.len() < 2 { return false; }
    V_INV[verhoeff_run(digits, 0)] == 0
}

/// The rail-network check for a number: which network, if any.
/// The kernel's cn_rail_valid: Luhn over the FULL 20 digits, and the
/// inner check over CN_RAIL_LEN-1 = 19 digits (the 18-base + the inner
/// digit) — NOT the full 20. This matches cn_check.c exactly.
pub fn rail_network(number: &str) -> Option<RailNetwork> {
    let b = number.as_bytes();
    if b.len() != 20 || !b.iter().all(|c| c.is_ascii_digit()) { return None; }
    if !luhn_valid(b) { return None; }
    if b[0] == b'0' { return None; }
    let inner19 = &b[..19]; // the kernel's CN_RAIL_LEN-1 slice
    let (net, inner_ok) = if number.starts_with("8809") {
        (RailNetwork::Dragon, mod1110_valid(inner19))
    } else if number.starts_with("8829") {
        (RailNetwork::Phoenix, damm_valid(inner19))
    } else if number.starts_with("8849") {
        (RailNetwork::Thunderbird, verhoeff_valid(inner19))
    } else {
        return None;
    };
    if inner_ok { Some(net) } else { None }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct RailsRegistry {
    pub owner: AccountId,
    /// number -> card
    pub cards: LookupMap<String, RailCard>,
    /// holder -> card numbers
    pub holder_cards: LookupMap<String, Vec<String>>,
}

#[near_bindgen]
impl RailsRegistry {
    #[init]
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            cards: LookupMap::new(b"c".to_vec()),
            holder_cards: LookupMap::new(b"h".to_vec()),
        }
    }

    /// Mint a card: derive the number deterministically from the
    /// holder's account + generation, in one call. The holder is the
    /// signer ("the name on the card is the public key").
    pub fn mint_card(&mut self, network: RailNetwork) -> RailCard {
        let holder = env::predecessor_account_id().to_string();
        let gen = self.next_generation(&holder);
        // derive: hash(holder || network-prefix || generation), then
        // unbiased decimal digits, then the network's inner check + Luhn
        let number = self.derive_number(network, &holder, gen);
        self.register_card(RailCard {
            number: number.clone(),
            network,
            holder: holder.clone(),
            kind: "person".into(),
            parent: None,
            generation: gen,
            state: "active".into(),
        });
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"zxv_rails\",\"version\":\"1.0.0\",\"event\":\"card_minted\",\"data\":{{\"number\":\"{}\",\"network\":\"{:?}\"}}}}",
            number, network
        ));
        self.cards.get(&number).cloned().unwrap()
    }

    /// A card's record (gas-free view).
    pub fn card(&self, number: String) -> Option<RailCard> {
        self.cards.get(&number).cloned()
    }

    /// All cards of a holder (gas-free view).
    pub fn cards_of(&self, holder: String) -> Vec<String> {
        self.holder_cards.get(&holder).cloned().unwrap_or_default()
    }

    /// Verify a number: which network it belongs to, if any (gas-free).
    pub fn verify_number(&self, number: String) -> Option<RailNetwork> {
        rail_network(&number)
    }

    /// NEP-171 read surface: the card as a non-transferable token.
    pub fn nft_token(&self, token_id: String) -> Option<RailCard> {
        self.cards.get(&token_id).cloned()
    }

    /// NEP-171: cards are NON-TRANSFERABLE. Always fails.
    pub fn nft_transfer(&self) -> String {
        "rail cards cannot be transferred".to_string()
    }

    // ---- internals ----

    fn next_generation(&self, holder: &str) -> u32 {
        self.holder_cards.get(holder).map(|v| v.len() as u32).unwrap_or(0)
    }

    fn derive_number(&self, network: RailNetwork, holder: &str, gen: u32) -> String {
        // seed from sha256(holder || prefix || generation || attempt)
        for attempt in 0u32..64 {
            let seed_input = format!("{}:{}:{}:{}", holder, network.prefix(), gen, attempt);
            let digest = env::sha256(seed_input.as_bytes());
            // unbiased decimal digits: accept bytes < 250, take mod 10
            let mut digits = String::new();
            for &b in digest.iter() {
                if b < 250 {
                    digits.push(char::from(b'0' + (b % 10)));
                    if digits.len() == 11 { break; }
                }
            }
            if digits.len() != 11 { continue; }
            let candidate = format!("{}{}{}", network.prefix(), "001", digits); // 18 digits
            // append the network's inner check digit, then the Luhn digit
            let inner = self.inner_check_digit(network, &candidate);
            let with_inner = format!("{}{}", candidate, inner as char); // 19 digits
            let luhn = self.luhn_digit(&with_inner);
            let number = format!("{}{}", with_inner, luhn as char); // 20 digits
            env::log_str(&format!("RAIL_DIAG: net={:?} candidate={} number={}", network, candidate, number));
            // defense in depth: the derived number must be a valid rail
            // number (Luhn + the network's inner check) BEFORE it is
            // ever registered — a bad derivation is refused, not stored
            if rail_network(&number) == Some(network) && !self.cards.contains_key(&number) {
                return number;
            }
        }
        env::panic_str("rail number derivation exhausted");
    }

    fn inner_check_digit(&self, network: RailNetwork, digits: &str) -> u8 {
        let b = digits.as_bytes();
        match network {
            RailNetwork::Dragon => {
                // MOD 11,10: c with (p + c) mod 10 == 1
                let p = mod1110_run(b);
                (((11 - p) % 10) as u8) + b'0'
            }
            RailNetwork::Phoenix => {
                // the kernel's cn_damm_digit: the interim digit IS the
                // check (the table has a zero diagonal)
                let mut interim = 0usize;
                for &d in b { interim = DAMM[interim][(d - b'0') as usize] as usize; }
                interim as u8 + b'0'
            }
            RailNetwork::Thunderbird => {
                // the kernel's cn_verhoeff_digit: vh_inv[verhoeff_run(...,1)]
                let c = verhoeff_run(b, 1);
                (V_INV[c] + b'0')
            }
        }
    }

    fn luhn_digit(&self, digits: &str) -> u8 {
        // the kernel's cn_luhn_digit: (10 - luhn_sum(digits,len,TRUE)) % 10
        let b = digits.as_bytes();
        let s = luhn_sum(b, true);
        (((10 - s) % 10) as u8) + b'0'
    }

    fn register_card(&mut self, card: RailCard) {
        self.cards.insert(card.number.clone(), card.clone());
        let mut list = self.holder_cards.get(&card.holder).cloned().unwrap_or_default();
        list.push(card.number.clone());
        self.holder_cards.insert(card.holder.clone(), list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damm_table_matches_kernel() {
        // the Damm table is copied from cn_check.c — spot-check a row
        assert_eq!(DAMM[0], [0, 3, 1, 7, 5, 9, 8, 6, 4, 2]);
        assert_eq!(DAMM[9], [2, 5, 8, 1, 4, 3, 6, 7, 9, 0]);
    }

    #[test]
    fn luhn_known_values() {
        // 7992739871 + check digit 3 is the canonical Luhn example
        let n = b"79927398713";
        assert!(luhn_valid(n));
        assert!(!luhn_valid(b"79927398710"));
    }

    #[test]
    fn verhoeff_known_values() {
        // 236 -> valid Verhoeff (the classic example)
        assert!(verhoeff_valid(b"236"));
    }
}