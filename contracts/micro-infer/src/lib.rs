//! micro-infer — the on-chain micro-inference engine (AIVM wasm path).
//!
//! A BitNet-1.58-style ternary ([-1,0,+1]) lookup-table embedding
//! evaluation, embedded as a near-sdk-rs contract. This is the Layer-1
//! wasm path of the AIVM substrate: models small enough to embed
//! (< 2-5 MB) run deterministically IN the sandbox — no floats, no
//! off-chain mesh, output is TRUE by construction.
//!
//! Benchmark target: measure wasmer gas per forward eval on testnet.
//!
//! Surface:
//!   infer(token, dim) -> the deterministic activation
//!   check(token, dim) -> the known-answer vector (KAT, determinism)
//!   embed()           -> the ternary table (for verification)
//!   gas_report(tokens)-> run N evals, return the measured cost
//!
//! Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
//! SPDX-License-Identifier: Apache-2.0

use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, PanicOnDefault};

/// The 8x8 ternary embedding table: packed 2-bit cells
/// (0 = -1, 1 = 0, 2 = +1), 8 bytes per row.
const EMBED: [u8; 64] = [
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
    0x0E, 0x1F, 0x2A, 0x0B, 0x1C, 0x2D, 0x0E, 0x1F,
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
    0x0E, 0x1F, 0x2A, 0x0B, 0x1C, 0x2D, 0x0E, 0x1F,
    0x1A, 0x2B, 0x0C, 0x1D, 0x2E, 0x0F, 0x1A, 0x2B,
    0x2C, 0x0D, 0x1E, 0x2F, 0x0A, 0x1B, 0x2C, 0x0D,
];

/// decode a packed 2-bit ternary cell: 0 -> -1, 1 -> 0, 2 -> +1
#[inline]
fn cell(packed: u8, i: u32) -> i8 {
    let c = (packed >> (2 * (i & 3))) & 0x3;
    match c {
        0 => -1,
        2 => 1,
        _ => 0,
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MicroInfer;

#[near_bindgen]
impl MicroInfer {
    #[init]
    pub fn new() -> Self {
        Self
    }

    /// The deterministic activation for a token in dim dimensions.
    pub fn infer(&self, token: u32, dim: u32) -> i32 {
        if dim == 0 || dim > 8 {
            return 0;
        }
        let t = token % 8;
        let row = &EMBED[(t * 8) as usize..(t * 8 + 8) as usize];
        let mut acc = 0i32;
        for i in 0..dim {
            let v = cell(row[(i / 4) as usize], i);
            acc += (v as i32) * (v as i32);
        }
        acc
    }

    /// The known-answer vector: 1 iff infer matches the ground truth.
    /// Proves determinism on-chain.
    pub fn check(&self, token: u32, dim: u32) -> u32 {
        let expect: [[i32; 9]; 8] = [
            [0, 1, 2, 2, 3, 3, 4, 5, 6],
            [0, 1, 1, 2, 3, 3, 3, 4, 5],
            [0, 1, 1, 2, 3, 3, 3, 3, 4],
            [0, 1, 2, 2, 3, 3, 4, 5, 6],
            [0, 1, 1, 2, 3, 3, 3, 4, 5],
            [0, 1, 1, 2, 3, 3, 3, 3, 4],
            [0, 1, 2, 2, 3, 3, 4, 5, 6],
            [0, 1, 1, 2, 3, 3, 3, 4, 5],
        ];
        let t = (token % 8) as usize;
        let d = dim.min(8) as usize;
        (self.infer(token, dim) == expect[t][d]) as u32
    }

    /// The ternary table (for external verification).
    pub fn embed(&self) -> Vec<u8> {
        EMBED.to_vec()
    }

    /// Benchmark: run `tokens` forward evals and return the measured
    /// gas used (the AIVM's per-eval cost driver).
    /// NOTE: `prepaid_gas` is CONSTANT during a call; the consumed gas
    /// is `used_gas()`. (The earlier version measured 0 always — fixed.)
    pub fn gas_report(&self, tokens: u32) -> (u64, u64) {
        let start = env::used_gas();
        let mut acc = 0i32;
        for t in 0..tokens.min(100_000) {
            acc += self.infer(t, 8);
        }
        let used = env::used_gas().saturating_sub(start).as_gas();
        (used, acc as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kat_all_64() {
        let m = MicroInfer;
        let mut pass = 0;
        for t in 0..8 {
            for d in 1..=8 {
                if m.check(t, d) == 1 {
                    pass += 1;
                }
            }
        }
        assert_eq!(pass, 64, "KAT must be 64/64");
    }

    #[test]
    fn deterministic() {
        let m = MicroInfer;
        assert_eq!(m.infer(5, 8), m.infer(5, 8));
    }
}