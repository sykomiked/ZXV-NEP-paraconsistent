/* zxv_accounting_hook.c — the ZXV economy accounting hook for Xahau.
 *
 * Version-0 hook ABI EXACTLY as the guard_checker validates:
 *   - only hook() and cbak() functions, everything inline (no helpers)
 *   - accept/rollback take int64_t error_code
 *   - hook/cbak take uint32_t
 *   - no custom sections (cleaned after compile)
 *   - 32-byte state keys
 *
 * Records XRP settlement facts + the platform fee (0.5% rail) into hook
 * state on every payment; the machine polls and relays to NEAR/IPFS.
 */
#include <stdint.h>

extern int64_t accept(uint32_t read_ptr, uint32_t read_len, int64_t error_code);
extern int64_t rollback(uint32_t read_ptr, uint32_t read_len, int64_t error_code);
extern int64_t hook_account(uint32_t write_ptr, uint32_t write_len);
extern int64_t otxn_field(uint32_t write_ptr, uint32_t write_len, uint32_t field_id);
extern int64_t state(uint32_t write_ptr, uint32_t write_len, uint32_t kread_ptr, uint32_t kread_len);
extern int64_t state_set(uint32_t read_ptr, uint32_t read_len, uint32_t kread_ptr, uint32_t kread_len);
extern int32_t _g(uint32_t id, uint32_t maxiter);

#define GUARD(maxiter) _g(__LINE__, (maxiter) + 1)

#define sfAccount 0x80001UL
#define sfAmount 0x60001UL
#define sfDestination 0x80003UL
#define sfTransactionType 0x10002UL
#define ttPAYMENT 0
#define FEE_RAIL_BPS 50u
#define BPS_DENOM 10000u

int64_t hook(uint32_t ctx)
{
    if (ctx != 0)
        return accept(0, 0, 0);

    uint8_t hook_accid[20];
    hook_account((uint32_t)(uintptr_t)hook_accid, 20);

    uint32_t txn_type = 0;
    if (otxn_field((uint32_t)(uintptr_t)&txn_type, 4, sfTransactionType) != 4)
        return accept(0, 0, 0);
    if (txn_type != ttPAYMENT)
        return accept(0, 0, 0);

    uint8_t amt_buf[8];
    if (otxn_field((uint32_t)(uintptr_t)amt_buf, 8, sfAmount) != 8)
        return accept(0, 0, 0);

    /* XRP native amounts: sfAmount is a 64-bit field where the high
     * nibble 0x40 marks "native". The drops value is the low 56 bits.
     * (0x40 in the top byte is NOT a sign — negating it corrupts the
     * amount. Fixed: read the low 56 bits as the unsigned drops.) */
    uint64_t drops = 0;
    {
        int i;
        for (i = 1; GUARD(8), i < 8; ++i)
            drops = (drops << 8) | amt_buf[i];
    }

    uint8_t dst_accid[20];
    int inbound = 0;
    if (otxn_field((uint32_t)(uintptr_t)dst_accid, 20, sfDestination) == 20)
    {
        int eq = 1;
        int i;
        for (i = 0; GUARD(20), i < 20; ++i)
            if (dst_accid[i] != hook_accid[i])
            {
                eq = 0;
                break;
            }
        inbound = eq;
    }

    /* read + increment the tx counter (state key 0x03, 32 bytes) */
    {
        static const uint8_t KEY_CNT[32] = {0x03};
        uint8_t buf[8];
        uint64_t v = 0;
        if (state((uint32_t)(uintptr_t)buf, 8,
                  (uint32_t)(uintptr_t)KEY_CNT, 32) == 8)
        {
            int i;
            for (i = 0; GUARD(8), i < 8; ++i)
                v = (v << 8) | buf[i];
        }
        v++;
        {
            int i;
            for (i = 7; GUARD(8), i >= 0; --i)
            {
                buf[i] = (uint8_t) (v & 0xFF);
                v >>= 8;
            }
        }
        state_set((uint32_t)(uintptr_t)buf, 8,
                  (uint32_t)(uintptr_t)KEY_CNT, 32);
    }

    /* the amount rail: inbound to KEY_IN (0x01), outbound to KEY_OUT
     * (0x02) plus the 0.5% platform fee to KEY_FEE (0x04) */
    {
        static const uint8_t KEY_IN[32] = {0x01};
        static const uint8_t KEY_OUT[32] = {0x02};
        static const uint8_t KEY_FEE[32] = {0x04};
        uint8_t buf[8];
        uint64_t v = 0;
        if (state((uint32_t)(uintptr_t)buf, 8,
                  (uint32_t)(uintptr_t)(inbound ? KEY_IN : KEY_OUT), 32) == 8)
        {
            int i;
            for (i = 0; GUARD(8), i < 8; ++i)
                v = (v << 8) | buf[i];
        }
        v += drops;
        {
            int i;
            for (i = 7; GUARD(8), i >= 0; --i)
            {
                buf[i] = (uint8_t) (v & 0xFF);
                v >>= 8;
            }
        }
        state_set((uint32_t)(uintptr_t)buf, 8,
                  (uint32_t)(uintptr_t)(inbound ? KEY_IN : KEY_OUT), 32);

        if (!inbound)
        {
            /* platform fee: floor(drops * 50 / 10000), integer-exact */
            uint64_t fee = (drops / BPS_DENOM) * FEE_RAIL_BPS +
                           (drops % BPS_DENOM) * FEE_RAIL_BPS / BPS_DENOM;
            uint64_t fv = 0;
            if (state((uint32_t)(uintptr_t)buf, 8,
                      (uint32_t)(uintptr_t)KEY_FEE, 32) == 8)
            {
                int i;
                for (i = 0; GUARD(8), i < 8; ++i)
                    fv = (fv << 8) | buf[i];
            }
            fv += fee;
            {
                int i;
                for (i = 7; GUARD(8), i >= 0; --i)
                {
                    buf[i] = (uint8_t) (fv & 0xFF);
                    fv >>= 8;
                }
            }
            state_set((uint32_t)(uintptr_t)buf, 8,
                      (uint32_t)(uintptr_t)KEY_FEE, 32);
        }
    }

    return accept(0, 0, 0);
}

int64_t cbak(uint32_t ctx)
{
    return accept(0, 0, 0);
}