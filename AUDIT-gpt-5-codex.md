# hate.fun Smart Contract Security Review

## Scope
- On-chain program sources under `src/`
- Supporting documentation (`README.md`, `spell.md`, `QUICKSTART.md`, `SECURITY.md`, `TESTING.md`, `NATIVE-TESTING.md`, `INTEGRATION_TESTS.md`, `claude.md`)

## Methodology
- Manual review of on-chain logic and invariants
- Threat modeling of value flows and termination paths

## Findings Overview

| ID | Title | Severity | Status |
|----|-------|----------|--------|
| HF-01 | `close_bucket` treats up to 0.01 SOL escrow deposits as dust | High | ✅ RESOLVED |

## Detailed Findings

### HF-01: `close_bucket` treats legitimate escrow funds as dust ✅ RESOLVED
- **Severity:** High
- **Impact:** Creator can seize supporter deposits below 0.01 SOL before the first flip.
- **Description:** The `close_bucket` handler considers an escrow "empty" so long as its balance does not exceed the hard-coded `ESCROW_EMPTY_THRESHOLD` of 10,000,000 lamports (0.01 SOL) (`src/instructions/close_bucket.rs:66-73`). Any supporter deposit below that amount remains locked in the escrow, yet the creator may still close the bucket (because `last_flip_epoch == creation_epoch`) and sweep the funds (`src/instructions/close_bucket.rs:85-92`).
- **Exploit scenario:**
  1. Creator initializes a bucket with `initial_last_swap` above 0.01 SOL (typical values are 1 SOL).
  2. A supporter deposits 0.005 SOL (5,000,000 lamports) through `deposit_to_escrow`. The amount is below the flip threshold, so no flush occurs.
  3. Creator immediately calls `close_bucket`. Because the escrow balance is below the 0.01 SOL dust cap, the close succeeds and the creator receives the supporter's funds.
- **Recommendation:** Replace the fixed dust constant with the real rent baseline (e.g., `Rent::get().minimum_balance(0)`) or persist the escrow's initial balance during creation and require balances to match that value when closing. Only permit closure when both escrows equal their rent baseline, ensuring any non-rent lamports stay protected.
- **Resolution (October 24, 2025):**
  - **Fixed in:** `src/instructions/close_bucket.rs:70-75`
  - **Change:** Replaced hardcoded `ESCROW_EMPTY_THRESHOLD` (10,000,000 lamports) with dynamic rent-exempt minimum via `Rent::get()?.minimum_balance(0)` (~890,880 lamports for empty accounts)
  - **Impact:** Any deposit above rent-exempt minimum (even 1 lamport of user funds) now prevents bucket closure, protecting all legitimate deposits
  - **Testing:** All 5 integration tests pass, including `test_close_bucket_before_flip` which verifies the new behavior
  - **Formal Verification:** New Kani proof `verify_escrow_empty_check_fixed` mathematically proves the fix protects all deposits > rent_exempt_minimum

## Additional Observations
- Direct SOL transfers to PDAs (e.g., `main_bucket`) bypass program logic; documenting that these funds can be recovered by the creator before the first flip would help manage user expectations.
- ~~The program relies on unchecked lamport mutations; the explicit `overflow-checks = true` release profile mitigates wraparound, but adding helper functions that perform checked arithmetic around the unsafe blocks would harden the code further.~~ **✅ ADDRESSED:** All unsafe blocks now have comprehensive SAFETY documentation explaining justifications. Verified functions (`sum_balances`, `calculate_payout_distribution`) used throughout to prevent overflow.

## Formal Verification Results (Added: October 24, 2025)

### Kani Formal Verification: ✅ PASSED

Following the audit, formal verification was performed using Kani Rust Verifier 0.65.0 on critical arithmetic operations. **All 9 proof harnesses passed successfully** (updated October 24, 2025).

#### Verified Properties:

1. **Threshold Calculation (16 checks)** - Proves no overflow for `last_swap <= u64::MAX / 15000`
2. **Payout Distribution Conservation (26 checks)** - Mathematically proves `creator + claimer + winner = total` (no value loss/gain)
3. **Fee Validation (3 checks)** - Proves ≤20% enforcement
4. **Min Increase Validation (1 check)** - Proves 1-50% bounds enforcement
5. **Threshold Precision** - Proves no truncation exploits
6. **HF-01 Documentation (2 checks)** - Formally documents the 0.01 SOL threshold vulnerability
7. **Balance Summation** - Proves safe summation for realistic balances
8. **HF-01 Fix Verification (2 checks)** - **NEW:** Proves rent-exempt threshold protects all legitimate deposits
9. **Max Fee Calculation** - Proves 20% edge case is safe

#### Key Findings:

**✅ Arithmetic Safety:** All checked arithmetic operations proven safe from overflow/underflow for valid input ranges.

**✅ Value Conservation:** Payout distribution mathematically proven to conserve total value exactly.

**✅ HF-01 Confirmed:** Kani formally proves that balances ≤ 0.01 SOL (10,000,000 lamports) are treated as "empty", confirming the HF-01 vulnerability at the mathematical level.

**✅ HF-01 Fix Verified (NEW):** Kani proof `verify_escrow_empty_check_fixed` mathematically proves the fix using rent-exempt minimum protects ALL deposits above the rent-exempt threshold (even 1 lamport).

#### Verification Scope:

**What Kani Verified:**
- Pure arithmetic functions extracted from contract logic
- Overflow/underflow prevention
- Value conservation
- Parameter validation correctness

**What Kani Did NOT Verify:**
- Solana-specific operations (PDA derivation, CPI calls, account validation)
- Unsafe code blocks (lamports manipulation)
- Multi-instruction flows and integration behavior

These remain covered by integration tests and manual review.

#### Documentation:

- Full results: `VERIFICATION-RESULTS.md`
- Setup guide: `KANI-VERIFICATION.md`
- Verification code: `src/verification.rs`
- CI/CD: `.github/workflows/kani-verify.yml`

**Conclusion:** Formal verification provides mathematical proof of arithmetic correctness, complementing the manual audit and integration testing for comprehensive security assurance.

## Files Reviewed
- `src/lib.rs`
- `src/state.rs`
- `src/error.rs`
- `src/system_program.rs`
- `src/instructions/create_bucket.rs`
- `src/instructions/deposit_to_escrow.rs`
- `src/instructions/flush_escrow.rs`
- `src/instructions/claim_payout.rs`
- `src/instructions/close_bucket.rs`
- Documentation: `README.md`, `QUICKSTART.md`, `SECURITY.md`, `TESTING.md`, `NATIVE-TESTING.md`, `INTEGRATION_TESTS.md`, `spell.md`, `claude.md`

## Suggested Remediations
- Replace the fixed 0.01 SOL dust threshold in `close_bucket` with an equality check against the rent baseline (or persisted creation balance) so any supporter lamports keep the bucket open.
- Add regression coverage around `close_bucket` to enforce the corrected escrow-empty condition and guard against future regressions.
- Explicitly document or guard against direct SOL transfers to PDAs before the first flip so depositors understand how those funds can be reclaimed.

## Implementation Status (October 24, 2025)

### ✅ All Critical and Medium Issues Resolved

**HF-01 (High Severity):** FIXED - Now uses rent-exempt minimum instead of 0.01 SOL threshold
**Unsafe Operation Documentation:** ADDED - All unsafe blocks have comprehensive SAFETY comments
**Balance Validation:** ADDED - claim_payout validates sufficient funds before distribution
**Verified Functions:** DEPLOYED - sum_balances and calculate_payout_distribution used throughout
**Minimum Deposit:** ENFORCED - 1,000 lamport minimum prevents dust griefing
**Error Standardization:** COMPLETED - Consistent use of HateFunError

### Testing Status
- ✅ All 5 unit tests pass
- ✅ All 5 integration tests pass  
- ✅ 9 Kani formal verification proofs verified (including new HF-01 fix proof)

