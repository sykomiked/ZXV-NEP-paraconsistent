# The ZXV AIVM Substrate — decentralized AI compute on NEAR

**The decoupled verification substrate:** heavy tensor passes run
off-chain on the compute mesh; the on-chain kernel is the state
coordinator, memory-mapping supervisor, and deterministic dispute
arbiter. The physical constraint (validators cannot store 140 GB
weights or run FP matrix math in Wasmer) is bridged by *reference*, not
by upload.

```
┌────────────────────────────────────────────────────────────────────┐
│  Layer 3: Model Storage & Ingestion                                │
│  (Hugging Face · Arweave · IPFS · Filecoin · BitTorrent)           │
│  GGUF / SafeTensors / ONNX / ZEDEC-Wasm quantized weights          │
└───────────────────────────────────┬────────────────────────────────┘
                                    │ model manifests (SHA-256 + layer DAG)
                                    ▼
┌────────────────────────────────────────────────────────────────────┐
│  Layer 2: Off-Chain Heterogeneous Compute Mesh                     │
│  (8x Gaudi2 · H100 · TEEs/Shade nodes · decentralized GPUs)        │
│  forward inference (BitNet b1.58, Q4/Q8, wasm micro-models)        │
│  → execution trace + layer commitment hashes                       │
└───────────────────────────────────┬────────────────────────────────┘
                                    │ inference proofs & competing outputs
                                    ▼
┌────────────────────────────────────────────────────────────────────┐
│  Layer 1: On-Chain ZXV AI Substrate (NEAR mainnet)                 │
│  account 6ca890f… · freestanding wasm · the four-valued gate       │
│                                                                    │
│  [ Model Registry ]  [ Paraconsistent ALU ]  [ Execution Gate ]    │
│  model_id → weight   lpres_join isolates    payout/finality only   │
│  Merkle root         divergence into BOTH   on TRUE                │
└────────────────────────────────────────────────────────────────────┘
```

## The three substrate primitives (this repo)

| Primitive | Spec | Reference |
|---|---|---|
| Model ingestion | `docs/spec/model-manifest.md` — UMM: content-addressed weight trees, Merkle chunking, on-chain `model_id` (32 bytes) | — |
| Divergence arbitration | `docs/spec/inference-invariant.md` — MATCH→TRUE, DIVERGENT→BOTH, TIMEOUT→NEITHER; the dispute lifecycle | `lpres_join` (48-cell on-chain proof) |
| On-chain micro-inference | the wasm path for sub-5 MB models (BitNet 1.58 ternary, lookup embeddings) | `reference/micro_infer.c` → `micro_infer.wasm` (668 B) |

## The dual execution engine

**A. Native on-chain micro-inference (pure wasm).** Ternary / lookup
models (< 2-5 MB) live in wasm linear memory; the zero-libc C kernel
executes the forward pass with SIMD128 (`i8x16.add`). Deterministic by
construction (no floats) — the sandbox output IS the result.

Verified: `micro_infer.wasm` (668 B), KAT 64/64, 0.052 µs/eval,
~25 gas units per forward eval — **negligible on-chain cost.**

**B. Off-chain heavy mesh with optimistic execution.** Large models
(7B-70B+) run on the compute pool; the kernel is the hypervisor:
assigns the request, receives outputs + layer commitment hashes, and
arbitrates disagreements with the paraconsistent ALU.

## The paraconsistent consensus ALU

Two nodes submit divergent outputs for an intent (CUDA vs Gaudi vs
Metal drift):

1. The kernel executes `lpres_join` over the agreement predicate.
2. Divergence → `BOTH` (0x03): the dispute is recorded in the state
   journal as an isolated thread; the system keeps running.
3. Settlement pauses **only for the disputed intent**
   (`lpres_is_settleable(BOTH) == false`).
4. Resolution: a third node agrees (majority → TRUE), or a TEE
   signature attests one output (TRUE). Non-coercion holds: no join
   turns BOTH into TRUE without authenticated evidence (verified
   on-chain, 0 coerced cells).

## The developer experience

```
# 1. register a model into the substrate address space
near call 6ca890f… register_model \
  '{"model_uri":"ipfs://Qm…","arch":"bitnet-1.58b","merkle_root":"0x4a7f…"}'

# 2. invoke an execution instance
near call 6ca890f… run_inference \
  '{"model_id":"0x4a7f…","prompt_tokens":[128,54,981]}'
```

## Status

- Specs: model-manifest.md + inference-invariant.md — **authored**.
- Micro-inference wasm: **built + verified** (668 B, KAT 64/64,
  benchmarked). Deploy to a testnet satellite when funded to measure
  wasmer gas on-chain.
- Gate: live on NEAR (48-cell proof).
- The compute mesh (Gaudi2 etc.) is the off-chain fabric this spec
  coordinates; its nodes submit commitments this kernel arbitrates.