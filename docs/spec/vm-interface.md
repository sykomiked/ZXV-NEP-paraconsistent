# The ZXV Virtual Machine — the OS interface on NEAR

**How the whole operating system lives on NEAR as a series of smart
contracts, with an interface automated protocols detect and adapt to
immediately — the way wallets auto-detect NEP-141 tokens.**

The ZXV VM is not one contract. It is the **kernel (the live
contract) + the satellite subsystems (registry, oracle, bus, shield) +
the boot manifest (IPFS)** — presented as a single, self-describing
machine through a discovery interface any agent can probe in one call.

## 1. The OS metaphor → the contract map

| OS concept | ZXV VM piece | Where it lives |
|---|---|---|
| CPU / ALU | the paraconsistent gate (`lpres_join`) | kernel (live) |
| Memory | content-addressed state + IPFS boot manifest | kernel + armada |
| Filesystem | the ledger (vouchers, V-Bills, tributes) | kernel (live) |
| System calls | the 27 exported methods | kernel (live) |
| Boot sector | the world-registry contract | satellite |
| IPC | the ipc-bus contract | satellite |
| Security / vault | the execution-shield contract | satellite |
| Drivers / services | oracle-gateway, model registry | satellite |

## 2. The discovery interface (the "auto-detect" surface)

Any protocol — an indexer, a wallet, a DAO, an agent — detects the ZXV
VM with **one probe**: call `vm_info()` on the registry contract.

```rust
// the ZXV VM discovery trait (mirrors NEP-141's ft_metadata pattern)
pub trait ZxvVm {
    /// One call reveals the whole OS: version, kernel, capability list.
    fn vm_info(&self) -> VmInfo;

    /// A named capability: its contract, entry methods, and ABI.
    fn vm_capability(&self, name: String) -> Option<Capability>;

    /// The full capability table (the syscall directory).
    fn vm_capabilities(&self) -> Vec<String>;
}

pub struct VmInfo {
    pub name: String,            // "ZXV Virtual Machine"
    pub version: String,         // "1.0.0"
    pub kernel: AccountId,       // the root contract
    pub code_hash: String,       // the kernel's verified hash
    pub boot_manifest: String,   // the IPFS CID of the boot image
    pub capabilities: Vec<String>, // ["gate","ledger","oracle","bus","shield","registry"]
}

pub struct Capability {
    pub name: String,
    pub contract: AccountId,     // where the subsystem lives
    pub entry: Vec<String>,      // the methods a protocol calls
    pub status: String,          // "live" | "authored" | "deployed"
}
```

### The probe (what a protocol actually runs)

```
1. vm_info()               -> the whole OS in one call
2. vm_capability("gate")   -> kernel + lpres_join (the ALU)
3. vm_capability("shield") -> the vault + attempt_payout
4. vm_capability("oracle") -> the dispute arbiter + assert
5. vm_capability("bus")    -> the IPC + publish/subscribe
```

One `vm_info()` call tells a protocol: the kernel, the code hash, the
boot manifest CID, and every subsystem with its contract address and
entry methods. **Adaptation is a lookup, not an integration project.**

## 3. The 27 kernel syscalls (the system-call table)

The kernel contract is the CPU; these are its syscalls:

| Class | Syscalls | Purpose |
|---|---|---|
| Boot | `zxv_boot`, `migrate` | init + layout upgrade |
| Economy | `voucher_issue/transfer/redeem`, `econ_issue`, `vbill_issue/redeem`, `fountain_grant` | the ledger |
| Logic | `lpres_join`, `lpres_verdicts` | the paraconsistent ALU |
| Ledger views | `ledger_state`, `chain_verify`, `economy`, `econ_state` | state (gas-free) |
| Token | `ft_metadata`, `ft_total_supply`, `ft_balance_of`, `ft_transfer`, `ft_transfer_call`, `ft_resolve_transfer`, `storage_deposit`, `storage_balance_of` | NEP-141 |
| SCP-144 | `scp144_bind` | the lattice binding |
| Probes | `read_probe`, `state_diag`, `zxv_announce` | forensics |

## 4. The adaptation protocol (how protocols catch on)

An automated protocol adapts in four steps, all gas-free reads:

```
STEP 1  PROBE    call vm_info() on the registry
        → learns: kernel address, code hash, boot CID, capability list

STEP 2  RESOLVE  for each needed capability, call vm_capability(name)
        → learns: subsystem contract + entry methods + status

STEP 3  REGISTER storage_deposit on the kernel (NEP-141, free)
        → becomes a first-class account of the OS

STEP 4  USE      gate: lpres_join(op,a,b)         (verdicts)
                 ledger: ft_transfer / voucher_*   (value)
                 shield: attempt_payout(state)     (TRUE-only money)
                 oracle: assert/query              (dispute arbiter)
                 bus: publish/attest/consume       (messaging)
```

Merit-based: the discovery surface is public, gas-free, and
self-describing — a protocol that finds the kernel (via
`ft_metadata.reference`) or the registry (via the NEP) learns the whole
OS in one call. No advertising, no integration contract, no SDK
lock-in: the interface IS the adapter.

## 5. Why protocols adapt immediately

1. **One probe, whole OS** — `vm_info()` is the entire manual.
2. **Standard patterns** — the discovery mirrors NEP-141
   (`ft_metadata`); the ledger is NEP-141; the deposit/registration is
   NEP-145-style. Familiar to every NEAR developer.
3. **Gas-free discovery** — probing costs nothing, so every indexer
   and agent can afford to look.
4. **Stable identity** — the kernel account + code hash are the
   machine's ID; the boot manifest CID is content-verified.
5. **The gate is the differentiator** — no other NEAR system offers a
   four-valued, TRUE-only-settles arbitration surface. Protocols that
   need safe multi-agent settlement find it here and nowhere else.

## 6. Reference implementation

- `contracts/world-registry` — the boot sector implementing the
  discovery trait (vm_info / vm_capability / vm_capabilities).
- `docs/spec/adaptation-protocol.md` — the four-step probe sequence.
- The kernel: live on mainnet
  (`6ca890f766fdaeccd6c934da325f5f06ebc63ca9be4f5f1b44d0686af93c64a8`,
  code hash `6N7d4cDC…`, 27 syscalls).