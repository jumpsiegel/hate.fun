# hate.fun Smart Contract Security Audit

**Program ID:** Not deployed to mainnet yet
**Audit Date:** October 2025
**Auditor:** Grok CLI
**Contract Version:** 0.1.0
**Framework:** Pinocchio v0.9.2 on Solana

## Executive Summary

The hate.fun program implements a competitive fundraising mechanism on Solana where two parties compete by depositing SOL to control the payout destination. The contract has been formally verified using Kani for critical arithmetic operations and includes comprehensive testing.

**Overall Assessment:** The contract demonstrates good security practices with formal verification, but contains one known critical vulnerability (HF-01) that allows creators to seize small legitimate deposits. Several medium-risk issues exist around lamports handling and potential edge cases.

**Key Findings:**
- **Critical:** 1 (HF-01 deposit seizure vulnerability)
- **High:** 0
- **Medium:** 4 (lamports handling, validation gaps)
- **Low:** 2 (code quality, documentation)
- **Informational:** 3 (design choices, optimizations)

## Critical Issues

### 1. HF-01: Creator Can Seize Legitimate Deposits (CRITICAL)

**Location:** `src/instructions/close_bucket.rs:67-69`

**Description:**
The `close_bucket` instruction uses a hardcoded threshold of 10,000,000 lamports (0.01 SOL) to determine if escrows are "empty". This allows the creator to close a bucket and seize any deposits up to 0.01 SOL before the first flip occurs.

```rust
const ESCROW_EMPTY_THRESHOLD: u64 = 10_000_000; // 0.01 SOL
if escrow_a_balance > ESCROW_EMPTY_THRESHOLD || escrow_b_balance > ESCROW_EMPTY_THRESHOLD {
    return Err(HateFunError::EscrowsNotEmpty.into());
}
```

**Impact:**
- Creator can steal deposits ≤ 0.01 SOL
- Affects legitimate users who deposit small amounts
- Permanent fund loss for victims

**Exploit Scenario:**
1. Creator creates bucket
2. User deposits 0.005 SOL to escrow
3. Creator immediately calls `close_bucket` before first flip
4. Creator receives all funds including the user's deposit
5. User loses 0.005 SOL

**Mitigation:**
Use actual rent-exempt minimum instead of arbitrary threshold:

```rust
let rent = Rent::get()?;
let rent_exempt_minimum = rent.minimum_balance(0);
if escrow_a_balance > rent_exempt_minimum || escrow_b_balance > rent_exempt_minimum {
    return Err(HateFunError::EscrowsNotEmpty.into());
}
```

**Status:** ✅ **FORMALLY VERIFIED** - Kani proof `verify_escrow_empty_check_hf01` mathematically proves this vulnerability exists.

**Recommendation:** Fix before mainnet deployment.

## High Issues

No high-severity issues found.

## Medium Issues

### 2. Insufficient Balance Validation in claim_payout (MEDIUM)

**Location:** `src/instructions/claim_payout.rs:86-102`

**Description:**
The `claim_payout` instruction collects all funds into the bucket account first, then distributes them. However, it doesn't validate that the bucket account has sufficient lamports to cover all the fee distributions before performing the unsafe transfers.

**Code:**
```rust
// First, collect all funds to bucket account
unsafe {
    *bucket_account.borrow_mut_lamports_unchecked() += balances[0] + balances[1] + balances[2];
    // ... zero out other accounts
}

// Now distribute from bucket account
unsafe {
    *bucket_account.borrow_mut_lamports_unchecked() -= creator_cut;
    *creator.borrow_mut_lamports_unchecked() += creator_cut;
    // ... similar for claimer and winner
}
```

**Impact:**
If the total calculation is somehow incorrect or if there's an underflow, the bucket account could end up with insufficient funds, causing the transaction to fail unexpectedly or potentially causing incorrect distributions.

**Mitigation:**
Add balance validation before transfers:

```rust
let bucket_balance = bucket_account.lamports();
if bucket_balance < total {
    return Err(HateFunError::InsufficientFunds.into());
}
```

### 3. Arithmetic Overflow in close_bucket Total Calculation (MEDIUM)

**Location:** `src/instructions/close_bucket.rs:74-80`

**Description:**
The total calculation in `close_bucket` uses `checked_add` but could still overflow if all PDA balances are extremely large (though practically unlikely).

**Code:**
```rust
let total = main_balance
    .checked_add(bucket_balance)
    .and_then(|sum| sum.checked_add(escrow_a_balance))
    .and_then(|sum| sum.checked_add(escrow_b_balance))
    .ok_or(HateFunError::Overflow)?;
```

**Impact:**
If overflow occurs, the instruction fails with `Overflow` error, preventing bucket closure. While the funds remain safe, this creates a denial-of-service condition.

**Mitigation:**
Use the verified `sum_balances` function:

```rust
use crate::verification::sum_balances;

let balances = [main_balance, bucket_balance, escrow_a_balance, escrow_b_balance];
let total = sum_balances(&balances).ok_or(HateFunError::Overflow)?;
```

### 4. Unsafe Lamports Operations Lack Atomicity Guarantees (MEDIUM)

**Location:** All instruction handlers using `unsafe` blocks

**Description:**
All lamports transfers use `unsafe` operations that directly manipulate account balances. While the operations are performed in sequence, there's no atomicity guarantee if the instruction were to be interrupted (though Solana transactions are atomic).

**Impact:**
The code is safe because Solana transactions are atomic, but the use of `unsafe` without proper justification could mask future bugs.

**Recommendation:**
Add comments justifying the `unsafe` usage and consider if the operations can be made safer.

### 5. Missing Validation for Zero Amount Deposits (MEDIUM)

**Location:** `src/instructions/deposit_to_escrow.rs:22-24`

**Description:**
The `deposit_to_escrow` instruction rejects zero-amount deposits but doesn't validate against extremely small amounts that could be used for griefing.

**Code:**
```rust
if amount == 0 {
    return Err(ProgramError::InvalidInstructionData);
}
```

**Impact:**
Users could deposit dust amounts (< rent-exempt minimum) that complicate bucket closure and waste blockchain space.

**Mitigation:**
Add minimum deposit validation:

```rust
if amount == 0 || amount < 1000 { // Minimum 0.000001 SOL
    return Err(ProgramError::InvalidInstructionData);
}
```

## Low Issues

### 6. Inconsistent Error Types (LOW)

**Location:** Various locations

**Description:**
Some errors use `ProgramError` while others use custom `HateFunError`. This creates inconsistency in error reporting.

**Examples:**
- `deposit_to_escrow.rs:24`: `ProgramError::InvalidInstructionData` for zero amount
- `create_bucket.rs:44`: `HateFunError::FeesTooHigh` for fee validation

**Impact:**
Minor inconsistency in error codes returned to users.

**Recommendation:** Standardize on custom error types for all domain-specific errors.

### 7. Missing Documentation for Security-Critical Constants (LOW)

**Location:** `src/instructions/close_bucket.rs:67`

**Description:**
The `ESCROW_EMPTY_THRESHOLD` constant lacks documentation explaining why 0.01 SOL was chosen and references to the vulnerability.

**Recommendation:**
Add documentation:

```rust
/// Threshold for considering escrow "empty" during bucket closure.
/// WARNING: This value enables the HF-01 vulnerability - deposits up to this amount
/// can be seized by the creator. Should be changed to actual rent-exempt minimum.
/// See: verify_escrow_empty_check_hf01 proof for formal verification of this issue.
const ESCROW_EMPTY_THRESHOLD: u64 = 10_000_000; // 0.01 SOL - VULNERABLE
```

## Informational Issues

### 8. Griefing Feature is Intentional (INFO)

**Description:**
The contract allows perpetual griefing where users can repeatedly flip control to keep the pot growing. This is documented as an intentional feature.

**Location:** `spell.md`, `README.md`

**Assessment:** This is a design choice, not a vulnerability. The griefing increases total funds raised, which aligns with the contract's goal.

### 9. Creator Cannot Be A or B (INFO)

**Description:**
The contract prevents the creator from being one of the competing addresses.

**Code:** `create_bucket.rs:44-46`

**Assessment:** Good practice to prevent conflicts of interest. No security impact.

### 10. Formal Verification Coverage (INFO)

**Description:**
Critical arithmetic operations are formally verified using Kani.

**Coverage:**
- ✅ Threshold calculations (8 proofs)
- ✅ Fee distributions (value conservation)
- ✅ Input validation
- ✅ Overflow protection

**Assessment:** Excellent security practice. The contract uses mathematical proofs for critical operations.

## Code Quality Assessment

### Strengths
- **Formal Verification:** Uses Kani for mathematical proof of correctness
- **Comprehensive Testing:** Unit tests, integration tests, and formal verification
- **Clear Architecture:** Well-structured with separation of concerns
- **Error Handling:** Custom error types with descriptive messages
- **Documentation:** Extensive documentation and technical specifications

### Areas for Improvement
- **Unsafe Usage:** More justification needed for `unsafe` blocks
- **Constant Documentation:** Security-critical constants need better documentation
- **Error Consistency:** Standardize error type usage
- **Input Validation:** Add minimum deposit amounts

## Attack Vectors Considered

### 1. Reentrancy
**Status:** Not Applicable
**Reason:** Solana programs don't support reentrancy due to transaction atomicity.

### 2. Integer Overflow
**Status:** ✅ Protected
**Reason:** All arithmetic uses `checked_*` operations or verified functions.

### 3. Access Control Bypass
**Status:** ✅ Protected
**Reason:** Proper signer validation and PDA ownership checks.

### 4. Race Conditions
**Status:** ✅ Protected
**Reason:** Solana transaction atomicity prevents race conditions.

### 5. Griefing Attacks
**Status:** ✅ Intentional
**Reason:** Documented as feature - increases total funds raised.

### 6. Front-running
**Status:** Not Applicable
**Reason:** No time-sensitive operations that could be front-run.

## Recommendations

### Immediate (Pre-Mainnet)
1. **Fix HF-01 Vulnerability** - Replace hardcoded threshold with rent-exempt minimum
2. **Add Balance Validation** - In `claim_payout` before distributions
3. **Use Verified Functions** - Replace manual arithmetic with verified functions where possible

### Short-term
1. **Add Minimum Deposit** - Prevent dust deposits
2. **Improve Documentation** - Document security-critical constants
3. **Standardize Errors** - Use consistent error types

### Long-term
1. **Security Audit** - Professional third-party audit before mainnet
2. **Bug Bounty** - Consider launching bug bounty program
3. **Monitoring** - Implement on-chain monitoring for unusual activity

## Testing Recommendations

### Additional Test Cases Needed
1. **HF-01 Exploit Test** - Verify deposits < 0.01 SOL can be seized
2. **Large Balance Test** - Test with balances near u64::MAX
3. **Edge Case Epochs** - Test claim_payout at exact epoch boundaries
4. **Dust Deposit Test** - Test behavior with very small deposits

### Fuzz Testing
Consider using cargo-fuzz to test instruction parsing and arithmetic edge cases.

## Conclusion

The hate.fun contract demonstrates sophisticated use of formal verification and testing methodologies. The critical HF-01 vulnerability must be fixed before mainnet deployment, but the overall architecture shows good security practices.

**Final Score:** 7/10
- **Security:** 6/10 (HF-01 vulnerability drags score down)
- **Code Quality:** 8/10
- **Testing:** 9/10
- **Documentation:** 8/10

**Recommendation:** Fix HF-01 and conduct professional security audit before mainnet deployment.

---

**Audit completed by:** Grok CLI
**Date:** October 2025
**Contact:** For questions about this audit, refer to the hate.fun project repository.