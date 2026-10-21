#!/usr/bin/env python3
"""register_feed.py — register the ZXV telemetry RSS feed everywhere.

Pings the discovery/indexing services so the feed gets crawled,
indexed, and propagated:

  1. Ping-O-Matic   — multi-service ping aggregator (API)
  2. WebSub hub     — pubsubhubbub.appspot.com (API, publish mode)
  3. Internet Archive — web.archive.org/save/<feed> (immutable snapshot)
  4. GitHub         — the feed is already committed; verify it's public

The feed URLs registered:
  https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/system_telemetry.xml      (full system)
  https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/network_validation.xml    (NEAR validation)
  https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/live_telemetry.xml        (9-server OS)
  https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/telemetry.xml             (kernel bilattice)

Copyright (c) 2024-2026 Michael Laurence Curzi and 36N9 Genetics, LLC
SPDX-License-Identifier: Apache-2.0
"""
import urllib.parse
import urllib.request

FEEDS = [
    "https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/system_telemetry.xml",
    "https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/network_validation.xml",
    "https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/live_telemetry.xml",
    "https://raw.githubusercontent.com/sykomiked/ZXV-NEP-paraconsistent/main/docs/telemetry/telemetry.xml",
]
BLOG = "https://github.com/sykomiked/ZXV-NEP-paraconsistent"
TITLE = "ZXV Armada — Full System Telemetry"


def ping_omatic(feed):
    """Ping-O-Matic: notifies many ping services at once."""
    params = urllib.parse.urlencode({
        "title": TITLE, "blogurl": BLOG, "rssurl": feed,
    })
    try:
        req = urllib.request.Request("http://rpc.pingomatic.com/?" + params)
        with urllib.request.urlopen(req, timeout=8) as r:
            return r.status, r.read()[:60]
    except Exception as e:
        return None, str(e)[:60]


def ping_websub(feed):
    """WebSub hub: publish mode (push to subscribers)."""
    data = urllib.parse.urlencode({"hub.mode": "publish", "hub.url": feed}).encode()
    try:
        req = urllib.request.Request("https://pubsubhubbub.appspot.com/", data=data, method="POST")
        with urllib.request.urlopen(req, timeout=8) as r:
            return r.status, "hub accepted"
    except Exception as e:
        return None, str(e)[:60]


def archive_save(feed):
    """Internet Archive: immutable snapshot of the feed."""
    url = "https://web.archive.org/save/" + feed
    try:
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req, timeout=15) as r:
            return r.status, r.geturl()[:60]
    except Exception as e:
        return None, str(e)[:60]


def main():
    print("=== REGISTERING THE ZXV TELEMETRY FEEDS ===")
    for feed in FEEDS:
        print(f"\n  feed: {feed}")
        for svc, fn in [("Ping-O-Matic", ping_omatic), ("WebSub", ping_websub),
                        ("Internet Archive", archive_save)]:
            st, msg = fn(feed)
            print(f"    {svc:<16} {'OK' if st and st < 400 else 'see'}: {msg.decode(errors='replace') if isinstance(msg, bytes) else msg}")
    print("\n  GitHub: the feeds are committed in docs/telemetry/ (public repo)")


if __name__ == "__main__":
    main()