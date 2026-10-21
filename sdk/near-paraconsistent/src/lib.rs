//! near-paraconsistent — the four-valued paraconsistent gate for NEAR.
//!
//! A zero-dependency crate that hides the raw bitwise math of the ZXV
//! kernel behind idiomatic Rust. Drop it into any near-sdk-rs contract
//! to make settlement safe by construction: **only `TRUE` settles**;
//! contradictions (`BOTH`) are recorded, never coerced, never a panic.
//!
//! ```
//! use near_paraconsistent::{Gate, LogicState};
//!
//! let state = Gate::kjoin(LogicState::True, LogicState::False);
//! assert_eq!(state, LogicState::Both);
//!
//! if !state.is_settleable() {
//!     // hold the settlement — no panic, no gas-burning rollback
//! }
//! ```
//!
//! Semantics match the ZXV kernel's `lpres_join` (`kjoin = (a|b)&3`,
//! chain rank `FALSE < NEITHER < BOTH < TRUE` for conjoin/disjoin).
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0
#![no_std]

use core::ops::BitOr;

/// The four foundation states of the paraconsistent gate.
///
/// | State     | Value | Meaning                        |
/// |-----------|-------|--------------------------------|
/// | `Neither` | `0x00` | no evidence yet                |
/// | `True`    | `0x01` | attested, agreed               |
/// | `False`   | `0x02` | explicitly refuted             |
/// | `Both`    | `0x03` | contradiction retained         |
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

impl LogicState {
    /// The raw byte value (0x00..0x03) — the kernel's wire format.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Decode from the kernel's raw byte.
    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v & 0x03 {
            0x00 => Some(Self::Neither),
            0x01 => Some(Self::True),
            0x02 => Some(Self::False),
            _ => Some(Self::Both),
        }
    }

    /// The settlement invariant: **only `TRUE` is settleable.**
    ///
    /// `BOTH` and `NEITHER` must never authorize financial settlement;
    /// `FALSE` explicitly refuses.
    #[inline]
    pub const fn is_settleable(self) -> bool {
        matches!(self, Self::True)
    }
}

impl BitOr for LogicState {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_u8(self.as_u8() | rhs.as_u8()).expect("two bits or'd stay in 0x00..0x03")
    }
}

/// The gate: the kernel's four-valued operations.
pub struct Gate;

impl Gate {
    /// The kernel's chain rank: `FALSE < NEITHER < BOTH < TRUE`.
    /// This differs from the raw byte order (0x00..0x03), so conjoin
    /// and disjoin use this rank, not the byte values.
    #[inline]
    const fn rank(s: LogicState) -> u8 {
        match s {
            LogicState::False => 0,
            LogicState::Neither => 1,
            LogicState::Both => 2,
            LogicState::True => 3,
        }
    }

    #[inline]
    const fn from_rank(r: u8) -> LogicState {
        match r {
            0 => LogicState::False,
            1 => LogicState::Neither,
            2 => LogicState::Both,
            _ => LogicState::True,
        }
    }

    /// Knowledge join: `kjoin(a, b) = (a | b) & 3`.
    ///
    /// The core operation. `kjoin(TRUE, FALSE) = BOTH` — a contradiction
    /// is absorbed and isolated, never coerced, never a deadlock.
    #[inline]
    pub const fn kjoin(a: LogicState, b: LogicState) -> LogicState {
        LogicState::from_u8((a.as_u8() | b.as_u8()) & 0x03).expect("mask keeps 0x00..0x03")
    }

    /// Meet (chain rank: `FALSE < NEITHER < BOTH < TRUE`).
    /// The conservative join: the result is the lower-ranked of the two.
    #[inline]
    pub const fn conjoin(a: LogicState, b: LogicState) -> LogicState {
        let (x, y) = (Self::rank(a), Self::rank(b));
        let min = if x < y { x } else { y };
        Self::from_rank(min)
    }

    /// Join (existential): the result is the higher-ranked of the two.
    /// Never used for settlement.
    #[inline]
    pub const fn disjoin(a: LogicState, b: LogicState) -> LogicState {
        let (x, y) = (Self::rank(a), Self::rank(b));
        let max = if x > y { x } else { y };
        Self::from_rank(max)
    }
}

/// A guard helper for contract methods: evaluate evidence, and only
/// proceed when the result is settleable (`TRUE`). Contradictions are
/// held (returned as `Err`), never panicked.
///
/// This is the pattern for oracle-divergence protection: instead of
/// `env::panic_str` (gas-burning rollback), hold the state in register.
#[inline]
pub fn settle_or_hold(states: &[LogicState]) -> Result<LogicState, LogicState> {
    let mut acc = LogicState::Neither;
    for &s in states {
        acc = Gate::kjoin(acc, s);
    }
    if acc.is_settleable() {
        Ok(acc)
    } else {
        Err(acc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kjoin_true_false_is_both() {
        // the promoted on-chain proof: kjoin(TRUE, FALSE) = BOTH
        assert_eq!(Gate::kjoin(LogicState::True, LogicState::False), LogicState::Both);
    }

    #[test]
    fn kjoin_true_true_is_true() {
        assert_eq!(Gate::kjoin(LogicState::True, LogicState::True), LogicState::True);
    }

    #[test]
    fn both_is_absorbing() {
        // a majority cannot coerce a recorded contradiction
        let s = Gate::kjoin(Gate::kjoin(LogicState::True, LogicState::False), LogicState::True);
        assert_eq!(s, LogicState::Both);
    }

    #[test]
    fn only_true_settles() {
        assert!(LogicState::True.is_settleable());
        assert!(!LogicState::Neither.is_settleable());
        assert!(!LogicState::False.is_settleable());
        assert!(!LogicState::Both.is_settleable());
    }

    #[test]
    fn conjoin_chain_rank() {
        // FALSE < NEITHER < BOTH < TRUE
        assert_eq!(Gate::conjoin(LogicState::False, LogicState::True), LogicState::False);
        assert_eq!(Gate::conjoin(LogicState::Neither, LogicState::True), LogicState::Neither);
        assert_eq!(Gate::conjoin(LogicState::Both, LogicState::True), LogicState::Both);
    }

    #[test]
    fn disjoin_existential() {
        assert_eq!(Gate::disjoin(LogicState::False, LogicState::True), LogicState::True);
        assert_eq!(Gate::disjoin(LogicState::Neither, LogicState::True), LogicState::True);
    }

    #[test]
    fn settle_or_hold_holds_contradiction() {
        let r = settle_or_hold(&[LogicState::True, LogicState::False, LogicState::True]);
        assert_eq!(r, Err(LogicState::Both));
        // agreement settles
        let ok = settle_or_hold(&[LogicState::True, LogicState::True]);
        assert_eq!(ok, Ok(LogicState::True));
    }

    #[test]
    fn wire_roundtrip() {
        for v in [0x00u8, 0x01, 0x02, 0x03] {
            let s = LogicState::from_u8(v).unwrap();
            assert_eq!(s.as_u8(), v);
        }
    }
}