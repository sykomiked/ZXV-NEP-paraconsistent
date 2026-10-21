# Universal Model Manifest (UMM) — the content-addressed weight tree

**The standard for loading AI models into the ZXV AIVM substrate by
reference — never by uploading tensors into the NEAR trie.**

Models live in decentralized storage (Hugging Face, IPFS, Arweave,
Filecoin, BitTorrent). The on-chain kernel registers only a **32-byte
manifest hash** mapped to the model's weight tree, so a 140 GB model
costs the same storage rent as a URL.

---

## 1. The manifest header

A UMM is a compact JSON document that deterministically describes a
model. Its canonical serialization (sorted keys, no whitespace) is
SHA-256-hashed to produce the `model_id` registered on-chain.

```json
{
  "schema": "zxv.umm.v1",
  "arch": "bitnet-1.58b",
  "quant": "ternary",
  "layers": 16,
  "hidden": 1024,
  "weight_tree": {
    "root": "0x4a7f…",           // Merkle root of all layer chunks
    "chunk_size": 1048576,        // 1 MiB chunks
    "layers": [
      { "layer": 0, "cid": "Qm…", "sha256": "0x…" },
      { "layer": 1, "cid": "Qm…", "sha256": "0x…" }
    ]
  },
  "tokenizer": "ipfs://Qm…",
  "license": "apache-2.0"
}
```

### Required fields

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `"zxv.umm.v1"` (versioned) |
| `arch` | string | model family (Llama-3, DeepSeek, Qwen, bitnet-1.58b, custom) |
| `quant` | string | quantization: `Q4_K_M`, `INT8`, `ternary`, `fp16` |
| `layers` | uint | number of transformer/linear layers |
| `hidden` | uint | hidden dimension |
| `weight_tree.root` | bytes32 | Merkle root over all layer chunks |
| `weight_tree.chunk_size` | uint | chunk size in bytes (power of two) |
| `weight_tree.layers[]` | list | per-layer content addresses |
| `tokenizer` | string | content-addressed tokenizer URI |

### Canonical serialization

The manifest MUST be serialized with sorted keys and no insignificant
whitespace (`json.dumps(..., sort_keys=True, separators=(",", ":"))`),
then hashed: `model_id = SHA-256(canonical_manifest)`. The same
manifest on any machine yields the same `model_id` — the model is
content-addressed by its description.

## 2. The weight tree (Merkle chunking)

Large weight files (`.gguf`, `.safetensors`) are split into fixed
1 MiB chunks (configurable; power of two). Each chunk is content-
addressed on IPFS (`cid`) and hashed (`sha256`). The per-layer list
forms a Merkle tree whose root is `weight_tree.root`.

- **Verification:** given any chunk, recompute its hash and walk the
  tree to the root; a mismatch means the weights were tampered with and
  the model_id is rejected.
- **Partial loads:** a model can be loaded layer-by-layer (memory-
  mapped), so a 70B model can run on a node that only holds a shard.
- **Quantization is part of the address:** the same weights quantized
  two ways produce two different manifests and two different model_ids —
  no ambiguity about what is loaded.

## 3. On-chain registration

The kernel (or the model-registry subsystem) stores:

```
model_id (32 bytes) -> { manifest_sha256, weight_root, quant, arch, status }
```

| Status | Meaning |
|---|---|
| `REGISTERED` | manifest accepted, weights loadable by reference |
| `LOADED` | at least one compute node has fetched + verified the tree |
| `DISPUTED` | a weight-verification disagreement is open (BOTH) |
| `SUSPENDED` | verification failed — no inference authorized |

Registration is a rare, deliberate write (~storage for one row). No
tensor data ever enters the trie.

## 4. Model URI schemes

| Scheme | Meaning |
|---|---|
| `ipfs://<cid>` | content-addressed chunk (IPFS / Filebase / armada) |
| `hf://<repo>/<file>` | Hugging Face reference (resolved off-chain) |
| `ar://<txid>` | Arweave permanent storage |
| `bt://<infohash>` | BitTorrent swarm (chunked retrieval) |

The on-chain kernel never fetches these URIs itself; compute nodes do,
and they submit the chunk hashes back as evidence.

## 5. Relation to the AIVM layers

```
Layer 3 (storage):   GGUF / Safetensors on IPFS/HF/Arweave/BT
                     │  model manifests (SHA-256 + layer DAG)
Layer 2 (mesh):      8x Gaudi2 / H100 / TEE nodes fetch chunks,
                     run forward passes, submit layer commitment hashes
Layer 1 (on-chain):  model_id → weight_root (this manifest),
                     paraconsistent ALU, execution gate
```

The manifest is the seam between storage and compute: it lets the
on-chain kernel be a **memory-mapping supervisor** over models it never
holds.

## 6. Example (minimal)

```json
{"arch":"bitnet-1.58b","chunk_size":1048576,"hidden":1024,"layers":16,
 "license":"apache-2.0","quant":"ternary","schema":"zxv.umm.v1",
 "tokenizer":"ipfs://QmTok","weight_tree":{"chunk_size":1048576,
 "layers":[{"cid":"QmL0","layer":0,"sha256":"0x00…"}],
 "root":"0x4a7f…"}}
```

`model_id = SHA-256(above) = 0x…` — a 32-byte on-chain handle for the
entire model.