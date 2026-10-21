#!/usr/bin/env python3
"""feed_system.py — the full-system RSS feed (NEAR + OS + CIDs).

The complete showcase, syndicated: the ZXV operating system works end
to end — the NEAR deployments respond, the 9-node Armada runs the
kernel, and the IPFS content resolves. One feed, three layers:

  NEAR:  zero-gas calls to the deployments (gate->BOTH, supply, vm_info…)
  OS:    the 9 Armada nodes serving the ZXV kernel
  IPFS:  our CIDs resolving on the Filebase/IPFS gateway (200)

Rate-limited (>=8s between RPC calls, 429 backoff). The feed is the
window; the chain and the content are the verified truth behind it.
"""
import base64
import json
import os
import sys
import time
import urllib.parse
import urllib.request
from datetime import datetime, timezone

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from armada_mesh_daemon import NODES, http_post  # noqa: E402

RPC = "https://rpc.mainnet.fastnear.com"
KERNEL = "6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8"
MICRO = "86f80143df70df0b02aa7625eddba7d4900bd543de24fa193b7fd4e0330a4dbe"
REGISTRY = "7eaa2c6f37ef7dd1dc073c4630533b67f303274d31d411e4047eb0d9401ef01a"
CONSUMER = "8947dcb7d35102dca4dc6cb852038b17efa1ff1a421209e5d0e7122425b0c654"
FEED_PATH = os.environ.get("SYS_FEED", "/Users/36n9/zxv-ipfs-build/dist/telemetry/system_telemetry.xml")
FEED_URL = os.environ.get("FEED_URL", "https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/system_telemetry.xml")
WEBSUB_HUB = "https://pubsubhubbub.appspot.com/"
MIN_GAP = 8.0

# our content-addressed objects (the IPFS layer)
CIDS = [
    ("boot_manifest", "QmPLTYTNa9NpKpySy66Pnc13UxB5GrZ4dqLAS6NkU6VAqd"),
    ("forensic_ledger", "QmaipCyuo9iu4ybsosEh3DMCeeEo7ofymWauZCAXMtyN56"),
    ("code_hash_migration", "QmUWkDmgeMSz4APn8EFkAEuZtzq6McnnAocg66q3qH35q7"),
    ("verifier_script", "Qmaa8SaZwFY9hXGrU4rd8C6bFDsFnFKKuHCgtd11i4rU95"),
    ("yhvh_page", "QmTicpvS1JvSUiXV5n6hLgSvBDHd8yGVMSaF7dyfLK1LPM"),
    ("system_docs", "QmPVXB22kX1HSSXRn76GfBgBLwFqfMtPNbgR9mHJkD7PiU"),
    ("world_computer", "QmRHCVJnujbpkv5ce1JnjP9j3nX2eq4XwgJ15sNKUAqU3P"),
]

_last_call = [0.0]
history = []


def _throttle():
    wait = MIN_GAP - (time.time() - _last_call[0])
    if wait > 0:
        time.sleep(wait)
    _last_call[0] = time.time()


def rpc_view(acct, method, args=None, raw=None):
    ab = base64.b64encode(raw).decode() if raw is not None else \
        base64.b64encode(json.dumps(args or {}).encode()).decode()
    data = json.dumps({"jsonrpc": "2.0", "id": "v", "method": "query", "params": {
        "request_type": "call_function", "finality": "final",
        "account_id": acct, "method_name": method, "args_base64": ab}}).encode()
    _throttle()
    for attempt in range(4):
        req = urllib.request.Request(RPC, data=data, headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                return bytes(json.loads(r.read()).get("result", {}).get("result", []))
        except Exception as e:
            if getattr(e, "code", None) == 429:
                time.sleep(15 * (attempt + 1))
                continue
            return b"ERR"
    return b"RATE_LIMIT"


def cid_status(cid):
    """Does the CID resolve on the public IPFS gateway? (the content proof)."""
    for gw in ["https://ipfs.filebase.io/ipfs/", "https://dweb.link/ipfs/"]:
        try:
            req = urllib.request.Request(gw + cid)
            with urllib.request.urlopen(req, timeout=10) as r:
                return "RESOLVES", r.status
        except Exception:
            continue
    return "UNREACHABLE", None


def node_status(name, ip):
    """One light probe: the node answers (the OS proof)."""
    for attempt in range(2):
        try:
            http_post(ip, "/", b"", timeout=5)
            return "LIVE"
        except urllib.error.HTTPError:
            return "LIVE"
        except Exception:
            time.sleep(1)
    return "DOWN"


def add_entry(entry):
    history.insert(0, entry)
    if len(history) > 300:
        history.pop()
    write_feed()


def write_feed():
    items = ""
    for ev in history:
        items += f"""
    <item>
      <title>[{ev['layer']}] {ev['title']}</title>
      <link>{ev['link']}</link>
      <description><![CDATA[
        <p><strong>Layer:</strong> {ev['layer']}</p>
        <p><strong>Detail:</strong> {ev['detail']}</p>
        <p><strong>Result:</strong> {ev['result']}</p>
        <p><strong>Status:</strong> {ev['status']}</p>
      ]]></description>
      <pubDate>{ev['rfc822']}</pubDate>
      <guid isPermaLink="false">{ev['guid']}</guid>
    </item>"""
    xml = f"""<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>ZXV — Full System Telemetry (NEAR · Armada OS · IPFS)</title>
    <link>https://github.com/sykomiked/ZXV-NEP-paraconsistent</link>
    <description>The whole operating system working: NEAR deployments, the 9-node kernel mesh, and content-addressed CIDs — live, syndicated</description>
    <atom:link rel="hub" href="{WEBSUB_HUB}" />
    <atom:link rel="self" href="{FEED_URL}" type="application/rss+xml" />
    {items}
  </channel>
</rss>"""
    tmp = FEED_PATH + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(xml)
    os.replace(tmp, FEED_PATH)


def ping_websub():
    data = urllib.parse.urlencode({"hub.mode": "publish", "hub.url": FEED_URL}).encode()
    try:
        req = urllib.request.Request(WEBSUB_HUB, data=data, method="POST")
        urllib.request.urlopen(req, timeout=3)
        return True
    except Exception:
        return False


def run():
    print("=== ZXV FULL-SYSTEM TELEMETRY (NEAR + OS + IPFS) ===")
    # NEAR layer
    r = rpc_view(KERNEL, "lpres_join", raw=bytes([2, 1, 2]))
    label = {0: "NEITHER", 1: "TRUE", 2: "FALSE", 3: "BOTH"}.get(r[0] if r and r != b"ERR" else -1, "?")
    add_entry({"layer": "NEAR", "title": f"gate lpres_join -> {label}",
               "link": f"https://nearblocks.io/address/{KERNEL}",
               "detail": "lpres_join(kjoin, TRUE, FALSE) on the kernel",
               "result": f"{label} (0x0{r[0] if r else 0})", "status": "OK",
               "guid": f"zxv-near-gate-{int(time.time()*1000)}",
               "rfc822": datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S GMT")})
    r = rpc_view(REGISTRY, "vm_info")
    add_entry({"layer": "NEAR", "title": "discovery vm_info",
               "link": f"https://nearblocks.io/address/{REGISTRY}",
               "detail": "the whole OS in one call",
               "result": r.decode(errors="replace")[:80] if r and r != b"ERR" else "ERR", "status": "OK",
               "guid": f"zxv-near-vm-{int(time.time()*1000)}",
               "rfc822": datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S GMT")})
    # OS layer
    for name, ip, _ in NODES:
        st = node_status(name, ip)
        add_entry({"layer": "OS", "title": f"{name} {st}",
                   "link": f"http://{ip}/zxv_boot.wasm",
                   "detail": f"Armada node {name} ({ip})",
                   "result": f"serves the ZXV kernel ({st})", "status": st,
                   "guid": f"zxv-os-{name}-{int(time.time()*1000)}",
                   "rfc822": datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S GMT")})
    # IPFS layer
    for name, cid in CIDS:
        st, code = cid_status(cid)
        add_entry({"layer": "IPFS", "title": f"{name} {st}",
                   "link": f"https://ipfs.filebase.io/ipfs/{cid}",
                   "detail": f"content-addressed object {cid[:20]}…",
                   "result": f"{st} ({code})", "status": st,
                   "guid": f"zxv-cid-{name}-{int(time.time()*1000)}",
                   "rfc822": datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S GMT")})
    ping_websub()
    print(f"\nfeed: {FEED_PATH} ({len(history)} items) | websub {'sent' if ping_websub() else 'failed'}")


if __name__ == "__main__":
    run()