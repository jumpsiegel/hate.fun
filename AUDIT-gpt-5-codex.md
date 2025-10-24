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
| HF-01 | `close_bucket` treats up to 0.01 SOL escrow deposits as dust | High | Unresolved |

## Detailed Findings

### HF-01: `close_bucket` treats legitimate escrow funds as dust
- **Severity:** High
- **Impact:** Creator can seize supporter deposits below 0.01 SOL before the first flip.
- **Description:** The `close_bucket` handler considers an escrow "empty" so long as its balance does not exceed the hard-coded `ESCROW_EMPTY_THRESHOLD` of 10,000,000 lamports (0.01 SOL) (`src/instructions/close_bucket.rs:66-73`). Any supporter deposit below that amount remains locked in the escrow, yet the creator may still close the bucket (because `last_flip_epoch == creation_epoch`) and sweep the funds (`src/instructions/close_bucket.rs:85-92`).
- **Exploit scenario:**
  1. Creator initializes a bucket with `initial_last_swap` above 0.01 SOL (typical values are 1 SOL).
  2. A supporter deposits 0.005 SOL (5,000,000 lamports) through `deposit_to_escrow`. The amount is below the flip threshold, so no flush occurs.
  3. Creator immediately calls `close_bucket`. Because the escrow balance is below the 0.01 SOL dust cap, the close succeeds and the creator receives the supporter’s funds.
- **Recommendation:** Replace the fixed dust constant with the real rent baseline (e.g., `Rent::get().minimum_balance(0)`) or persist the escrow’s initial balance during creation and require balances to match that value when closing. Only permit closure when both escrows equal their rent baseline, ensuring any non-rent lamports stay protected.

## Additional Observations
- Direct SOL transfers to PDAs (e.g., `main_bucket`) bypass program logic; documenting that these funds can be recovered by the creator before the first flip would help manage user expectations.
- The program relies on unchecked lamport mutations; the explicit `overflow-checks = true` release profile mitigates wraparound, but adding helper functions that perform checked arithmetic around the unsafe blocks would harden the code further.

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
