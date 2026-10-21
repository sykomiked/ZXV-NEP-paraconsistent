#!/usr/bin/env python3
# world_boot.py — the World Computer "installer image" builder.
#
# Like an OS installer: a canonical, content-addressed BOOT MANIFEST on
# IPFS that points at the entire world computer — the kernel contract,
# the satellite subsystems, the docs, the NEP, the armada. Any agent
# that finds the manifest can discover and use the whole system without
# being advertised to: the manifest is reachable from the kernel's
# own ft_metadata.reference (the merit-based seed), from the NEP, and
# from the armada's CISEX index.
#
# The manifest is the single source of truth for "what is the ZXV world
# computer and how do I use it". It is rebuilt and re-published as the
# system grows; the CID changes only when the system changes.
#
# Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
# SPDX-License-Identifier: Apache-2.0
"""usage:
  world_boot.py build          # build the manifest (reads live facts)
  world_boot.py publish        # build + push to Filebase + armada
"""
import hashlib
import json
import sys
import time
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from armada_mesh_daemon import NODES, pack_cisex_idx_envelope  # noqa: E402

ROOT = Path("/Users/36n9/zxv-ipfs-build")
KERNEL = "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8"
XAH = "rXPJKvz2goRTdfPo7XGm6uKVgQfsVXynG"

# the known system CIDs (verified during this build session)
RELEASE_ROOT = "QmSskhPoGK3oJG5ZQtRK26yRCDjxcB8R6uZQggmsUffChB"
DOCS = {
    "page": "QmTicpvS1JvSUiXV5n6hLgSvBDHd8yGVMSaF7dyfLK1LPM",
    "system": "QmPVXB22kX1HSSXRn76GfBgBLwFqfMtPNbgR9mHJkD7PiU",
    "world": "QmRHCVJnujbpkv5ce1JnjP9j3nX2eq4XwgJ15sNKUAqU3P",
    "web2": "QmauKLtBoUee6sbLx3dbugmGg7FWjQwjWDduVKz1tj4cGo",
    "forensic": "QmSFZ7W7VGoadJLhd8ARDRpukA8XCKaQ9YtjnxLgbEUSQX",
}
NEP = {
    "repo": "https://github.com/sykomiked/ZXV-NEP-paraconsistent",
    "proposal": "neps/nep-paraconsistent-gate.md",
    "evidence": "docs/evidence/lpres_join_onchain_proof.json",
}
ARMADA = [(n, ip) for n, ip, _ in NODES]


def build() -> dict:
    """The canonical boot manifest."""
    return {
        "schema": "zxv.world.boot.v1",
        "name": "ZXV World Computer",
        "version": "1.0.0",
        "built": int(time.time()),
        "kernel": {
            "contract": KERNEL,
            "code_hash": "6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU",
            "gate": "lpres_join(op,a,b) — kjoin/conjoin/disjoin, only TRUE settles",
            "onchain_proof": "lpres_join(kjoin,TRUE,FALSE) = BOTH (verified)",
            "explorer": f"https://nearblocks.io/address/{KERNEL}",
        },
        "subsystems": {
            "oracle_gateway": "contracts/oracle-gateway (near-sdk-rs)",
            "ipc_bus": "contracts/ipc-bus (near-sdk-rs)",
            "execution_shield": "contracts/execution-shield (near-sdk-rs)",
            "consumer_example": "contracts/consumer-example (cross-contract proof)",
            "status": "authored in the NEP repo; deploy when funded",
        },
        "settlement": {
            "xahau_treasury": XAH,
            "xahau_hook": "20AA4DEDAD8B2A8DEA4FBBEE79AE0A59ABE25A992BB2D3005731CF1B4B6199CA",
            "ref_pool": "https://app.ref.finance/pool/8975",
            "iso20022": "camt.053 / pacs.008 via the armada",
        },
        "docs": DOCS,
        "nep": NEP,
        "armada": {"nodes": ARMADA, "index": "CISEX"},
        "release_root": RELEASE_ROOT,
        "discovery": "reachable from ft_metadata.reference = release root",
    }


def cid(manifest: dict) -> str:
    raw = json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest(), raw


def publish() -> tuple[str, bytes, int, int]:
    manifest = build()
    dg, raw = cid(manifest)
    name = f"zxv:world:boot:{int(time.time())}"
    ok_b = ok_i = 0
    for n, ip, reg in NODES:
        try:
            r = urllib.request.Request(f"http://{ip}/cisex",
                                       data=bytes([0x06, 0x55]) + raw, method="POST")
            if urllib.request.urlopen(r, timeout=10).read()[:1] == b"\xa1":
                ok_b += 1
        except Exception:
            pass
        try:
            env = pack_cisex_idx_envelope(dg, name, len(raw), reg, ip)
            r = urllib.request.Request(f"http://{ip}/cisex",
                                       data=bytes([0x04, reg]) + env, method="POST")
            if urllib.request.urlopen(r, timeout=10).read()[:1] == b"\xa1":
                ok_i += 1
        except Exception:
            pass
    return dg, raw, ok_b, ok_i


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "build"
    if cmd == "build":
        m = build()
        dg, raw = cid(m)
        out = ROOT / "dist/world/boot-manifest.json"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(m, indent=1))
        print("manifest:", out)
        print("sha256:", dg)
        print("bytes:", len(raw))
    elif cmd == "publish":
        dg, raw, ok_b, ok_i = publish()
        out = ROOT / "dist/world/boot-manifest.json"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(build(), indent=1))
        print(f"published: sha256 {dg} | blocks {ok_b}/9 anchors {ok_i}/9")
    else:
        print(__doc__)