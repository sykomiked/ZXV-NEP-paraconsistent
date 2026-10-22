//! verdict-guard — the scenic bypass for the deployed gate's kjoin defect.
//!
//! The deployed gate (`lpres_join` op 2, kjoin) returns TRUE for some
//! non-TRUE pairs because `kjoin(a,b) = (a|b)&3` does not range-check
//! inputs: `kjoin(0, 5)` -> `(0|5)&3 = 1` = TRUE. Verified live on
//! mainnet: `lpres_join [2,0,5] -> 0x01 (TRUE)`.
//!
//! The fix (per the rails brief, C6): combine evidence with
//! `lpres_conjoin` ONLY, and range-check every input to 0..3 before
//! combining. This satellite is that guard — the rails call THIS, never
//! the raw gate op 2. The promoted kernel hash is untouched.
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, PanicOnDefault};

/// The four foundation states (0..3).
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Copy,
    Debug, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
#[borsh(use_discriminant = true)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

impl LogicState {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// The kernel's ACTUAL lpres_conjoin table (from lpres.c), copied
/// rule-for-rule so the guard agrees byte-for-byte:
///   FALSE dominates; TRUE is identity; NEITHER dominates BOTH
///   (a gap + a contradiction must never grant authority).
const fn lpres_conjoin(a: LogicState, b: LogicState) -> LogicState {
    let (x, y) = (a.as_u8(), b.as_u8());
    // FALSE (0x02) dominates
    if x == 0x02 || y == 0x02 {
        return LogicState::False;
    }
    // TRUE (0x01) is identity
    if x == 0x01 {
        return b;
    }
    if y == 0x01 {
        return a;
    }
    // NEITHER (0x00) dominates BOTH (0x03): a gap + a contradiction
    // must never grant authority
    if x == 0x00 || y == 0x00 {
        return LogicState::Neither;
    }
    LogicState::Both
}

/// The kernel's ACTUAL lpres_disjoin table (from lpres.c):
///   TRUE dominates; FALSE is identity; BOTH dominates NEITHER.
const fn lpres_disjoin(a: LogicState, b: LogicState) -> LogicState {
    let (x, y) = (a.as_u8(), b.as_u8());
    // TRUE (0x01) dominates
    if x == 0x01 || y == 0x01 {
        return LogicState::True;
    }
    // FALSE (0x02) is identity
    if x == 0x02 {
        return b;
    }
    if y == 0x02 {
        return a;
    }
    // BOTH (0x03) dominates NEITHER (0x00)
    if x == 0x03 || y == 0x03 {
        return LogicState::Both;
    }
    LogicState::Neither
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct VerdictGuard {}

#[near_bindgen]
impl VerdictGuard {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    /// THE rails entry point: combine evidence with lpres_conjoin ONLY.
    /// Every input is range-checked to 0..3 BEFORE combining — the
    /// defect fix. Returns the verdict; TRUE means settle.
    pub fn conjoin(&self, inputs: Vec<u8>) -> LogicState {
        // range-check every input to 0..3 FIRST (the kjoin defect path
        // was out-of-range inputs sliding to TRUE via &3), then fold
        // with the kernel's conjoin. The fold starts from the FIRST
        // evidence — a neutral NEITHER accumulator would poison the
        // join (NEITHER dominates BOTH).
        let mut states = [LogicState::Neither; 32];
        let mut n = 0usize;
        for raw in inputs {
            let s = match raw {
                0x00 => LogicState::Neither,
                0x01 => LogicState::True,
                0x02 => LogicState::False,
                0x03 => LogicState::Both,
                _ => env::panic_str("verdict-guard: evidence out of range 0..3"),
            };
            states[n] = s;
            n += 1;
            if n >= states.len() {
                env::panic_str("verdict-guard: too many evidence");
            }
        }
        if n == 0 {
            return LogicState::Neither;
        }
        let mut acc = states[0];
        for i in 1..n {
            acc = lpres_conjoin(acc, states[i]);
        }
        acc
    }

    /// The existential join (never for settlement).
    pub fn disjoin(&self, inputs: Vec<u8>) -> LogicState {
        let mut acc = LogicState::Neither;
        for raw in inputs {
            let s = match raw {
                0x00 => LogicState::Neither,
                0x01 => LogicState::True,
                0x02 => LogicState::False,
                0x03 => LogicState::Both,
                _ => env::panic_str("verdict-guard: evidence out of range 0..3"),
            };
            acc = lpres_disjoin(acc, s);
        }
        acc
    }

    /// The settlement invariant: only TRUE settles.
    pub fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }

    /// Human-readable label.
    pub fn label(&self, state: LogicState) -> String {
        match state {
            LogicState::Neither => "NEITHER".into(),
            LogicState::True => "TRUE".into(),
            LogicState::False => "FALSE".into(),
            LogicState::Both => "BOTH".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_range_rejected() {
        // THE defect: [2,0,5] on the raw gate returned TRUE. The guard
        // must refuse out-of-range input (panic), never return TRUE.
        let g = VerdictGuard::new();
        let r = std::panic::catch_unwind(|| g.conjoin(vec![0, 5]));
        assert!(r.is_err(), "out-of-range input must be rejected");
    }

    #[test]
    fn conjoin_matches_kernel_table() {
        let g = VerdictGuard::new();
        // the kernel's table (from lpres.c):
        assert_eq!(g.conjoin(vec![1, 1]), LogicState::True);    // T,T -> T
        assert_eq!(g.conjoin(vec![1, 2]), LogicState::False);   // T,F -> F (F dominates)
        assert_eq!(g.conjoin(vec![0, 1]), LogicState::Neither); // N,T -> N (T identity)
        assert_eq!(g.conjoin(vec![0, 3]), LogicState::Neither); // N,B -> N (gap dominates)
        assert_eq!(g.conjoin(vec![2, 3]), LogicState::False);   // F,B -> F (F dominates)
        assert_eq!(g.conjoin(vec![3, 3]), LogicState::Both);    // B,B -> B
        // NEVER TRUE unless every input is TRUE:
        assert_eq!(g.conjoin(vec![0, 3, 1]), LogicState::Neither);
    }

    #[test]
    fn settle_only_on_true() {
        let g = VerdictGuard::new();
        assert!(g.is_settleable(LogicState::True));
        assert!(!g.is_settleable(LogicState::Both));
        assert!(!g.is_settleable(LogicState::Neither));
        assert!(!g.is_settleable(LogicState::False));
    }
}