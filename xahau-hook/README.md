# The ZXV Xahau accounting hook — deployed + patched

The treasury (`rXPJKvz2goRTdfPo7XGm6uKVgQfsVXynG`) carries a 5-hook
chain. The ZXV accounting hook is at **chain position 0** (the
always-executing slot) after research found position 4 never executes
on this account.

## The two fixes (from the 2026-10-20 debugging session)

1. **Position fix:** moved the hook from position 4 (never executed,
   0/100 txns) to position 0 via SetHook `4B89CB88…`. The chain now
   runs 0→4.

2. **Amount-negate bug (in the deployed wasm):** `i32.const 64 /
   i32.and / select` treated the native-flag bit (0x40) of every XRP
   amount as a SIGN, negating all amounts. Patched the wasm byte at
   offset 550 (LEB `c0 00` -> `00 00`) — see `patch_negate.py`.

3. **Deploy format (the temMALFORMED fix):** a SetHook with
   `CreateCode` MUST include `HookOn` (the payment-trigger mask) or the
   ledger rejects it. The original deploy had it; the first re-deploys
   omitted it.

## Deployed state (2026-10-20)

- SetHook `AFED8600…` tesSUCCESS — patched hook live at position 0
- New HookHash: `C0481700EA8CE85E00A50A4BB6A36F54F3B46EF19A7DC1FBF073D5E77B1E12E3`
- Definition: HookOn fires on Payment, namespace
  `4C079F14455EDE8E9C273C5F2377E0CBF04BB8EE4E3BA0A6ECCB2419A8D3565F`
  (SHA-256 of "zxv-accounting"), fee 2,333 drops
- The hook fires on payments (verified: 4-hook execution including it);
  on-chain state counters have a residual write-path issue (the type/
  amount read semantics in the 1,055-byte binary) that needs the Linux
  wasi-sdk build environment to fully resolve. Settlement accounting is
  live via the xahau_bridge daemon (tracks drops_in/drops_out).

## Files

- `zxv_accounting_hook.c` — the hook source (records XRP settlement +
  the 0.5% platform rail to hook state)
- `zxv_hook_fixed.wasm` — the deployed patched wasm (negate removed)
- `patch_negate.py` — the byte patch that removes the sign bug

## Build (needs the Xahau wasi-sdk, Linux)

The hook must compile with the Xahau wasi-sdk clang (Linux) — Homebrew
clang produces wasm the hook ABI rejects (temMALFORMED) unless the
`HookOn` field is included. The deployed binary is the reference.
