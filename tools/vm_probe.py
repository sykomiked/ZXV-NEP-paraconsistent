#!/usr/bin/env python3
# vm_probe.py — the ZXV VM discovery probe (what any automated protocol
# runs to detect and adapt to the OS). All gas-free reads.
#
# Steps: PROBE (vm_info) -> RESOLVE (vm_capability) -> REGISTER note
# -> USE (gate check). Mirrors docs/spec/adaptation-protocol.md.
#
# Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
# SPDX-License-Identifier: Apache-2.0
"""usage:
  vm_probe.py info            # PROBE: the whole OS in one call
  vm_probe.py gate            # USE: verify the gate live on the kernel
  vm_probe.py capabilities    # RESOLVE: the full capability table
  vm_probe.py adapt           # the full four-step adaptation flow
"""
import base64
import json
import sys
import urllib.request

KERNEL = "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8"
REGISTRY = KERNEL  # the registry contract (deploys to its own account when funded)
RPC = "https://rpc.mainnet.fastnear.com"
STATE = {0: "NEITHER", 1: "TRUE", 2: "FALSE", 3: "BOTH"}


def call(contract, method, args_b64=""):
    body = {"jsonrpc": "2.0", "id": "zxv-vm", "method": "query",
            "params": {"request_type": "call_function", "finality": "final",
                       "account_id": contract, "method_name": method,
                       "args_base64": args_b64}}
    req = urllib.request.Request(RPC, data=json.dumps(body).encode(),
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=20) as r:
        d = json.loads(r.read())
    res = d.get("result", {})
    if res.get("error"):
        raise RuntimeError(f"{method}: {res['error']}")
    return bytes(res.get("result", []))


def probe():
    """STEP 1: the whole OS in one call (kernel's vm surface is live;
    the registry's vm_info deploys with the satellites)."""
    # the kernel itself is the live part of the VM — probe its identity
    meta = call(KERNEL, "ft_metadata").decode()
    m = json.loads(meta)
    return {
        "name": "ZXV Virtual Machine",
        "version": "1.0.0",
        "kernel": KERNEL,
        "code_hash": "6N7d4cDCmrhi6CJYhcLLziZ5aYMJm6YmXBDAVK8cPAHU",
        "token_metadata": m,
        "capabilities": ["gate", "ledger", "registry", "oracle", "bus", "shield"],
        "note": "registry vm_info() goes live when the registry contract deploys; "
                "the kernel surface below is already live",
    }


def gate():
    """STEP 4a: verify the ALU live on the kernel."""
    args = base64.b64encode(bytes([2, 1, 2])).decode()  # kjoin(TRUE,FALSE)
    out = call(KERNEL, "lpres_join", args)
    v = out[0] if out else -1
    return {"call": "lpres_join(kjoin,TRUE,FALSE)", "result": STATE.get(v, str(v)),
            "pass": v == 3, "live": True}


def capabilities():
    """STEP 2: the capability table (from the spec; the on-chain
    vm_capability() deploys with the registry)."""
    return {
        "gate": {"contract": KERNEL, "entry": ["lpres_join", "lpres_verdicts"], "status": "live"},
        "ledger": {"contract": KERNEL, "entry": ["voucher_issue", "voucher_transfer", "voucher_redeem", "econ_issue", "ft_transfer"], "status": "live"},
        "registry": {"contract": "<registry>", "entry": ["vm_info", "vm_capability", "vm_capabilities"], "status": "live"},
        "oracle": {"contract": "oracle-gateway", "entry": ["assert", "query", "is_settleable"], "status": "authored"},
        "bus": {"contract": "ipc-bus", "entry": ["publish", "attest", "consume", "subscribe"], "status": "authored"},
        "shield": {"contract": "execution-shield", "entry": ["attempt_payout", "deposit"], "status": "authored"},
    }


def adapt():
    """The full four-step flow."""
    return {
        "step1_probe": probe(),
        "step2_resolve": capabilities(),
        "step3_register": "storage_deposit on the kernel (NEP-145, free) — "
                          "becomes a first-class VM account",
        "step4_use": gate(),
        "status": "ADAPTED" if gate()["pass"] else "PROBE FAILED",
    }


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "adapt"
    if cmd == "info":
        print(json.dumps(probe(), indent=1))
    elif cmd == "gate":
        print(json.dumps(gate(), indent=1))
    elif cmd == "capabilities":
        print(json.dumps(capabilities(), indent=1))
    else:
        print(json.dumps(adapt(), indent=1)[:1500])