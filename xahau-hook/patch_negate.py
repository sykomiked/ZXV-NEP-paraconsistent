#!/usr/bin/env python3
"""patch_negate.py — remove the amount-negate bug from the deployed hook.

The deployed hook treated the native-flag bit (0x40) of XRP amounts as a
sign (i32.const 64 / i32.and / select). This patches the wasm byte so the
amount stays positive.
usage: patch_negate.py <in.wasm> <out.wasm>
"""
import sys

def main():
    src, dst = sys.argv[1], sys.argv[2]
    raw = bytearray(open(src, "rb").read())
    # find i32.and (0x71) opcodes; the amount-negate one is preceded by
    # 'i32.const 64' encoded as 41 c0 00 (LEB with continuation byte)
    patched = 0
    i = 0
    while True:
        i = raw.find(bytes([0x71]), i)
        if i == -1:
            break
        if i >= 3 and raw[i-3] == 0x41 and raw[i-2] == 0xc0 and raw[i-1] == 0x00:
            raw[i-2] = 0x00  # 64 -> 0 (LEB value byte)
            raw[i-1] = 0x00  # continuation byte
            patched += 1
        i += 1
    open(dst, "wb").write(bytes(raw))
    print(f"patched {patched} negation(s): {src} -> {dst}")

if __name__ == "__main__":
    main()
