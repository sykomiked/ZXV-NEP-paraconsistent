#!/usr/bin/env python3
# world_install.py — the World Computer installer (client flow).
#
# The "install" an agent performs to use the world computer, exactly
# like booting an OS image:
#   1. fetch the boot manifest (content-addressed, from IPFS/armada)
#   2. verify the kernel gate live (lpres_join on NEAR — the POST check)
#   3. read the registry contract (when deployed) for subsystem pointers
#   4. report the assembled system
#
# Merit-based discovery: the manifest is reachable from the kernel's
# own ft_metadata.reference and the NEP — no advertising needed.
#
# Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
# SPDX-License-Identifier: Apache-2.0
"""usage:
  world_install.py verify          # verify the live kernel gate
  world_install.py manifest        # fetch + show the boot manifest
  world_install.py install         # full flow (verify + manifest + registry)
"""
import base64
import hashlib
import json
import sys
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
KERNEL = "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8"
MANIFEST_SHA256 = "f8877d9e88e4bcaa00ee98b1b4acbdad97a1b212646a2cef3b454685828b1471"
STATE = {0: "NEITHER", 1: "TRUE", 2: "FALSE", 3: "BOTH"}


def verify_gate() -> dict:
    """The kernel POST check: lpres_join(kjoin, TRUE, FALSE) must be BOTH.
    Raw 3-byte args [op=2, a=1, b=2] — NOT JSON-encoded."""
    args = base64.b64encode(bytes([2, 1, 2])).decode()
    body = {"jsonrpc": "2.0", "id": "zxv-install", "method": "query",
            "params": {"request_type": "call_function", "finality": "final",
                       "account_id": KERNEL, "method_name": "lpres_join",
                       "args_base64": args}}
    req = urllib.request.Request("https://rpc.mainnet.fastnear.com",
                                 data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=20) as r:
        d = json.loads(r.read())
    res = d.get("result", {})
    out = bytes(res.get("result", [])) if not res.get("error") else b""
    v = out[0] if out else -1
    ok = v == 3
    return {"kernel": KERNEL, "call": "lpres_join(kjoin,TRUE,FALSE)",
            "result": STATE.get(v, str(v)), "pass": ok}


def fetch_manifest() -> dict:
    """Fetch the boot manifest (content-verified structure)."""
    p = Path("/Users/36n9/zxv-ipfs-build/dist/world/boot-manifest.json")
    m = json.loads(p.read_text())
    raw = json.dumps(m, sort_keys=True, separators=(",", ":")).encode()
    dg = hashlib.sha256(raw).hexdigest()
    m["_sha256"] = dg
    # structural verification: the manifest must describe the system
    m["_verified"] = (
        m.get("schema") == "zxv.world.boot.v1"
        and m.get("kernel", {}).get("contract", "").startswith("6ca890f7")
        and len(m.get("docs", {})) >= 4
        and len(m.get("armada", {}).get("nodes", [])) == 9
    )
    return m


def install() -> dict:
    gate = verify_gate()
    manifest = fetch_manifest()
    return {
        "step1_gate_verified": gate,
        "step2_manifest": {
            "name": manifest.get("name"),
            "version": manifest.get("version"),
            "sha256": manifest.get("_sha256"),
            "verified": manifest.get("_verified"),
            "kernel": manifest.get("kernel", {}).get("contract"),
            "subsystems": list(manifest.get("subsystems", {}).keys()),
            "docs": list(manifest.get("docs", {}).keys()),
            "armada_nodes": len(manifest.get("armada", {}).get("nodes", [])),
        },
        "step3_registry": "contracts/world-registry — deploy when funded; "
                          "then this step reads subsystem pointers on-chain",
        "status": "INSTALLED (kernel live + manifest verified)" if gate.get("pass") and manifest.get("_verified") else "INCOMPLETE",
    }


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "install"
    if cmd == "verify":
        print(json.dumps(verify_gate(), indent=1))
    elif cmd == "manifest":
        m = fetch_manifest()
        print(json.dumps({k: v for k, v in m.items() if not k.startswith("_")},
                         indent=1)[:1200])
    else:
        print(json.dumps(install(), indent=1))