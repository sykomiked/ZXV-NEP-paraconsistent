# The ZXV VM Adaptation Protocol — how protocols catch on

**The four-step sequence an automated protocol (indexer, wallet, DAO,
agent) runs to detect and use the ZXV Virtual Machine — all gas-free,
all discoverable by merit, no integration contract.**

## The probe (the whole protocol in one page)

```
STEP 1  PROBE    vm_info()                     -> the whole OS
                {name, version, kernel, code_hash, boot_manifest,
                 capabilities: [gate, ledger, registry, oracle, bus, shield]}

STEP 2  RESOLVE  vm_capability("gate")         -> kernel + lpres_join (live)
                vm_capability("ledger")        -> kernel + vouchers (live)
                vm_capability("oracle")        -> oracle-gateway (authored)
                vm_capability("bus")           -> ipc-bus (authored)
                vm_capability("shield")        -> execution-shield (authored)

STEP 3  REGISTER storage_deposit on the kernel  -> first-class account
                (NEP-145-style, free, returns the account)

STEP 4  USE      gate:     lpres_join(op,a,b)      verdicts (gas-free)
                ledger:   ft_transfer / voucher_*  value
                shield:   attempt_payout(state)    TRUE-only money
                oracle:   assert / query            disputes
                bus:      publish / attest / consume
```

## 1. PROBE — one call, whole OS

```bash
curl -s https://rpc.mainnet.near.org -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"z","method":"query","params":{
       "request_type":"call_function","finality":"final",
       "account_id":"<registry>","method_name":"vm_info","args_base64":""}}'
```

Returns the kernel address, the verified code hash, the boot manifest
CID, and the six-capability directory. **A protocol that can read one
JSON object can use the entire OS.**

## 2. RESOLVE — capabilities are pointers, not integration

Each capability resolves to `{contract, entry[], status}`. The `status`
field is the honesty contract:

- `live` — deployed and running (gate, ledger, registry: the kernel's
  own surface).
- `authored` — written, verified locally, awaiting deployment (oracle,
  bus, shield: the satellites in this repo).

A protocol adapts to `live` capabilities immediately and polls
`status` for the rest — no breaking change when a satellite deploys;
the pointer just flips to `live`.

## 3. REGISTER — standard NEAR, familiar to every developer

`storage_deposit` on the kernel (NEP-141/145 pattern). The same call
any NEAR developer already makes to use a token. No custom SDK, no
lock-in.

## 4. USE — the syscalls

The kernel's 27 exports (see `docs/spec/vm-interface.md` §3) grouped
by class. The gate is the differentiator: `lpres_join` is the only
four-valued, TRUE-only-settles arbitration surface on NEAR. Protocols
that need safe multi-agent settlement find it here and nowhere else.

## Why it spreads without advertising

1. **Merit-based seed:** the kernel's `ft_metadata.reference` points at
   the release root → the boot manifest → the registry. Any indexer
   that reads token metadata finds the VM.
2. **Self-describing:** `vm_info()` is the entire manual; adaptation is
   a lookup.
3. **Familiar patterns:** NEP-141 metadata, NEP-145 storage,
   cross-contract promises — every NEAR developer already knows how to
   call it.
4. **Gas-free discovery:** probing costs nothing, so every automated
   protocol can afford to look.
5. **The differentiator:** the paraconsistent gate is genuinely unique
   and genuinely needed for agent/oracle divergence — the protocol that
   needs it finds it, uses it, and the discovery surface makes the
   spread automatic.

## Reference implementation

- `contracts/world-registry` — `vm_info` / `vm_capability` /
  `vm_capabilities` (the discovery surface).
- `tools/vm_probe.py` — the probe client any protocol can copy.
- `docs/spec/vm-interface.md` — the full syscall table.