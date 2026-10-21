// lpres.rs — the Paraconsistent Logic Gate, as a standalone Rust module.
//
// The reference implementation of the NEP in near-sdk-rs-compatible
// Rust, so any third-party agent can compile the logic locally without
// a cross-contract call. Mirrors `reference/lpres_gate.c` exactly.
//
// The four-valued model:
//   Neither = 0x00 — unobserved / null
//   True    = 0x01 — verified assertion
//   False   = 0x02 — refuted assertion
//   Both    = 0x03 — contradictory / disputed evidence
//
// Settlement invariant: only `True` authorizes. `Both` and `Neither`
// never authorize. A contradiction is recorded, never resolved by
// picking a side.
//
// Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
// SPDX-License-Identifier: Apache-2.0

use core::fmt;

/// The four foundation states.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogicState {
    Neither = 0x00,
    True = 0x01,
    False = 0x02,
    Both = 0x03,
}

impl LogicState {
    /// The project's chain rank: Neither(0) < False(1) < Both(2) < True(3).
    /// Meet/join use this rank, NOT the numeric code order.
    #[inline]
    pub fn rank(self) -> u8 {
        match self {
            LogicState::Neither => 0,
            LogicState::False => 1,
            LogicState::Both => 2,
            LogicState::True => 3,
        }
    }
}

impl fmt::Display for LogicState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogicState::Neither => "NEITHER",
            LogicState::True => "TRUE",
            LogicState::False => "FALSE",
            LogicState::Both => "BOTH",
        };
        f.write_str(s)
    }
}

/// The paraconsistent gate. This is the NEP's standard interface.
pub trait ParaconsistentGate {
    /// The knowledge join: agreement keeps the value; silence adds
    /// nothing; TRUE from one source + FALSE from another = BOTH.
    fn kjoin(&self, a: LogicState, b: LogicState) -> LogicState;

    /// The meet (conjoin): min on the chain rank. FALSE dominates,
    /// TRUE is identity, NEITHER absorbs BOTH.
    fn conjoin(&self, a: LogicState, b: LogicState) -> LogicState;

    /// The join (disjoin): max on the chain rank. TRUE dominates,
    /// FALSE is identity, BOTH absorbs NEITHER. EXISTENTIAL — never
    /// used to authorize settlement.
    fn disjoin(&self, a: LogicState, b: LogicState) -> LogicState;

    /// The settlement invariant: only TRUE authorizes.
    fn is_settleable(&self, state: LogicState) -> bool;

    /// Dispatch by op code (0 conjoin, 1 disjoin, 2 kjoin).
    fn join_op(&self, op: u8, a: LogicState, b: LogicState) -> LogicState {
        match op {
            0 => self.conjoin(a, b),
            1 => self.disjoin(a, b),
            _ => self.kjoin(a, b),
        }
    }
}

/// The reference implementation — the exact kernel gate.
#[derive(Copy, Clone, Debug, Default)]
pub struct NearParaconsistentGate;

impl ParaconsistentGate for NearParaconsistentGate {
    #[inline]
    fn kjoin(&self, a: LogicState, b: LogicState) -> LogicState {
        // (a | b) & 0x03 — the entire table.
        unsafe { core::mem::transmute::<u8, LogicState>((a as u8 | b as u8) & 0x03) }
    }

    #[inline]
    fn conjoin(&self, a: LogicState, b: LogicState) -> LogicState {
        if a.rank() <= b.rank() {
            a
        } else {
            b
        }
    }

    #[inline]
    fn disjoin(&self, a: LogicState, b: LogicState) -> LogicState {
        if a.rank() >= b.rank() {
            a
        } else {
            b
        }
    }

    #[inline]
    fn is_settleable(&self, state: LogicState) -> bool {
        state == LogicState::True
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use LogicState::*;

    const G: NearParaconsistentGate = NearParaconsistentGate;

    #[test]
    fn kjoin_table_matches_onchain() {
        // the 16-cell table, verified live on NEAR mainnet
        let expect = [
            [Neither, True, False, Both],
            [True, True, Both, Both],
            [False, Both, False, Both],
            [Both, Both, Both, Both],
        ];
        for (a, row) in expect.iter().enumerate() {
            for (b, want) in row.iter().enumerate() {
                let got = G.kjoin(transmute(a), transmute(b));
                assert_eq!(got, *want, "kjoin({},{})", a, b);
            }
        }
    }

    #[test]
    fn headline_the_gift() {
        assert_eq!(G.kjoin(True, False), Both); // contradiction retained
        assert!(!G.is_settleable(Both));
    }

    #[test]
    fn non_coercion_invariant() {
        // no kjoin containing FALSE or BOTH yields TRUE
        for a in [Neither, True, False, Both] {
            for b in [Neither, True, False, Both] {
                let r = G.kjoin(a, b);
                if a == False || a == Both || b == False || b == Both {
                    assert_ne!(r, True, "coerced: kjoin({},{})", a, b);
                }
            }
        }
    }

    #[test]
    fn commutativity_and_associativity() {
        for a in [Neither, True, False, Both] {
            for b in [Neither, True, False, Both] {
                assert_eq!(G.kjoin(a, b), G.kjoin(b, a));
                assert_eq!(G.conjoin(a, b), G.conjoin(b, a));
                assert_eq!(G.disjoin(a, b), G.disjoin(b, a));
                for c in [Neither, True, False, Both] {
                    assert_eq!(G.kjoin(G.kjoin(a, b), c), G.kjoin(a, G.kjoin(b, c)));
                }
            }
        }
    }

    #[test]
    fn both_is_absorbing_under_kjoin() {
        for a in [Neither, True, False, Both] {
            assert_eq!(G.kjoin(Both, a), Both);
            assert_eq!(G.kjoin(a, Both), Both);
        }
    }

    #[test]
    fn settlement_exactly_three_cells() {
        // settleable kjoin results: (T,T), (T,N), (N,T)
        let settleable = [
            (True, True),
            (True, Neither),
            (Neither, True),
        ];
        for a in [Neither, True, False, Both] {
            for b in [Neither, True, False, Both] {
                let s = G.is_settleable(G.kjoin(a, b));
                assert_eq!(s, settleable.contains(&(a, b)), "settleable kjoin({},{})", a, b);
            }
        }
    }

    #[test]
    fn conjoin_disjoin_rank_semantics() {
        // conjoin: FALSE dominates; disjoin: TRUE dominates (existential)
        assert_eq!(G.conjoin(False, True), False);
        assert_eq!(G.conjoin(Both, True), Both); // dispute not coerced
        assert_eq!(G.disjoin(True, False), True); // existential
        assert_eq!(G.disjoin(Both, True), True); // existential by design
    }

    // helper (safe transmute of codes 0..=3 into LogicState)
    fn transmute(v: usize) -> LogicState {
        match v {
            0 => Neither,
            1 => True,
            2 => False,
            _ => Both,
        }
    }
}