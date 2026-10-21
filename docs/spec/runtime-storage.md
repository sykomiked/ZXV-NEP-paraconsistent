# Runtime & Storage Invariants — the reference contract

**For Pagoda / runtime engineers.** This document specifies the
contract's storage layout, the gas/memory envelope of the gate, and the
monolith-vs-chunking decision — the three things runtime reviewers will
ask about when they inspect the ~898 KB state under key `"zxv:v1"`.

---

## 1. State layout (key `"zxv:v1"`)

The contract persists one monolithic serialized struct under a single
storage key. The layout (little-endian host struct, serialized as
written by `env_storage_write`):

| Offset (approx) | Field | Type | Notes |
|---|---|---|---|
| 0 | `magic` | u32 | `ZXV_NEAR_MAGIC` |
| 4 | `version` | u32 | `ZXV_NEAR_VERSION` |
| 8 | `booted` | u32 | 0/1 |
| 12 | `owner` | char[65] | the deploying account id |
| 77 | `ops` | u64 | monotonic operation counter |
| 85 | `tl` | `triple_ledger_t` | the voucher ledger (bounded arrays) |
| … | `vino` | `vino_ledger_t` | the settlement ledger + SHA-256 chain |
| … | `holders` | char[N][65] | holder registry |
| … | `num_holders` | u32 | |
| tail | `vbill_outstanding` | u64 | V-Bills issued |
| tail | `vbill_next_id` | u64 | next V-Bill id |

**Current size (verified on mainnet 2026-10-20):** 898,328 bytes
(storage_usage 950,272 bytes including the 45,586-byte code + account
overhead). `read_probe()` returns the stored length.

The struct is bounded by construction: `triple_ledger_t` and
`vino_ledger_t` are fixed-size arrays (guarded by
`ZXV_TL_MAX_ACCOUNTS` / `ZXV_TL_MAX_ENTRIES` / `ZXV_TL_MAX_VOUCHERS` and
`VINO_MAX_*`); nothing is heap-allocated. Deserialization is a bounded
memcpy from the register into the static `g_st`, then field-wise
validation.

## 2. Gas & memory envelope of the gate

The gate itself (`lpres_join`) is a **pure function**:

- **Compute:** a fixed 256-entry table over 4×4×3 cells, implemented as
  `(a|b)&3` for kjoin and rank min/max for conjoin/disjoin. Worst case:
  ~10 instructions. **No loops, no storage reads, no state mutation.**
  Gas cost is effectively the base view-call overhead (a view call
  carries no fee on NEAR; a change-call invoking the gate adds
  microseconds).
- **Memory:** the gate touches only 3 input bytes + 1 output byte on
  the wasm stack. It does **not** load `g_st` (the state-bearing
  methods do; the gate is pure and exported separately). Peak guest
  memory for the full contract is the wasm linear memory (~55 MB for
  the kernel engine; the NEAR contract itself uses far less — the
  static `g_st` is 898 KB inside the wasm memory).
- **Register usage:** the state methods read the register once
  (`register_len` → `read_register`) into `g_st`; the gate reads the
  input register (≤64 bytes) and writes the result register (1 byte).

## 3. Monolith vs. chunking — the decision and its justification

The contract persists state as **one key** rather than chunking into
many trie paths. The reasons:

1. **Atomicity.** The ledger's correctness depends on the SHA-256 chain
   and the invariant "nothing is written before it is checked" (the
   journal-then-commit rule). A single key write commits the whole
   state atomically; chunked writes would need a multi-key
   transactional protocol the runtime does not provide, risking partial
   states that break `chain_verify`.
2. **Determinism.** The kernel is integer-exact and order-dependent
   (the audit chain is a hash chain). Chunking would require a
   canonical chunk ordering and per-chunk hash links — added
   complexity with no correctness benefit for a bounded ~900 KB state.
3. **Chunk-proof overhead.** A single 898 KB value is one trie leaf;
   its inclusion proof is ~40 hashes regardless of size. Chunking into
   e.g. 64 × 14 KB leaves multiplies the proof by the number of chunks
   and adds inter-chunk dependency proofs. For a state this size, the
   monolith is *cheaper* to prove.
4. **Boundedness.** The state is bounded by fixed arrays (max ~900 KB
   by construction), so the single-key approach cannot grow
   unboundedly. The bound is a documented invariant.

**Trade-off (honest):** a monolithic value means every state-bearing
change re-serializes ~900 KB (the dominant gas cost of a change call),
and a future struct-layout change requires a migration (as happened
v10 → v11, handled by the `migrate()` method that reads the legacy
layout and zero-fills the new tail). Chunking would reduce per-write
bytes at the cost of atomicity and proof complexity. For a settlement
ledger, atomicity wins; the bound caps the cost.

## 4. Migration path (the v10→v11 lesson)

When the state struct changes size (fields appended), the stored state
written by an older layout is 16 bytes shorter. `state_load()` accepts
both `sizeof g_st` and `sizeof g_st - 16`, zero-fills the tail, and the
owner calls `migrate()` once to persist the new layout. This is the
documented, tested upgrade path (verified on mainnet 2026-10-19,
TX `EQxvVs6WWfUPe8iaru1gR3SaD7a9M3Usk96Bi73T6XzY`).

## 5. Verified numbers (mainnet, 2026-10-20)

| Quantity | Value |
|---|---|
| State size (`"zxv:v1"`) | 898,328 B |
| Storage usage (state + code + account) | 950,272 B |
| Code size (wasm) | 45,586 B |
| Gate worst-case compute | ~10 instructions (pure) |
| State-bearing change | one 898 KB re-serialize (dominant gas) |
| `migrate()` gas | 100 Tgas (measured: fits under 100 T, over 30 T) |