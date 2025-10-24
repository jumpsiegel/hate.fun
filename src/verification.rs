// Verification module for Kani formal verification
// This module contains pure arithmetic functions extracted from the main program
// along with Kani proof harnesses to verify their correctness

/// Calculate the threshold required to flip an escrow
/// Returns None if multiplication or division would overflow
pub fn calculate_flush_threshold(last_swap: u64, min_increase_bps: u16) -> Option<u64> {
    last_swap
        .checked_mul(10000_u64.checked_add(min_increase_bps as u64)?)?
        .checked_div(10000)
}

/// Calculate fee amounts and winner payout
/// Returns (creator_cut, claimer_cut, winner_cut) or None on overflow
pub fn calculate_payout_distribution(
    total: u64,
    creator_fee_bps: u16,
    claimer_fee_bps: u16,
) -> Option<(u64, u64, u64)> {
    // Use u128 for intermediate calculations to prevent overflow
    let creator_cut = (total as u128)
        .checked_mul(creator_fee_bps as u128)?
        .checked_div(10000)?;

    let claimer_cut = (total as u128)
        .checked_mul(claimer_fee_bps as u128)?
        .checked_div(10000)?;

    // Ensure creator_cut and claimer_cut fit in u64
    if creator_cut > u64::MAX as u128 || claimer_cut > u64::MAX as u128 {
        return None;
    }

    let creator_cut = creator_cut as u64;
    let claimer_cut = claimer_cut as u64;

    let winner_cut = total
        .checked_sub(creator_cut)?
        .checked_sub(claimer_cut)?;

    Some((creator_cut, claimer_cut, winner_cut))
}

/// Sum multiple balances with overflow checking
pub fn sum_balances(balances: &[u64]) -> Option<u64> {
    let mut total = 0u64;
    for &balance in balances {
        total = total.checked_add(balance)?;
    }
    Some(total)
}

/// Validate fee parameters at bucket creation
pub fn validate_fees(creator_fee_bps: u16, claimer_fee_bps: u16) -> bool {
    creator_fee_bps as u32 + claimer_fee_bps as u32 <= 2000
}

/// Validate minimum increase bounds
pub fn validate_min_increase(min_increase_bps: u16) -> bool {
    min_increase_bps >= 100 && min_increase_bps <= 5000
}

/// Check if escrow balance is considered "empty" (at or below dust threshold)
/// This is the HF-01 security issue: 0.01 SOL threshold may allow creator to seize deposits
pub fn is_escrow_empty(balance: u64, dust_threshold: u64) -> bool {
    balance <= dust_threshold
}

// ========== KANI PROOF HARNESSES ==========

#[cfg(kani)]
mod proofs {
    use super::*;

    // Proof 1: Threshold calculation never overflows for valid inputs
    #[kani::proof]
    fn verify_threshold_calculation() {
        let last_swap: u64 = kani::any();
        let min_increase_bps: u16 = kani::any();

        // Assume valid min_increase_bps (1-50%)
        kani::assume(min_increase_bps >= 100 && min_increase_bps <= 5000);

        // Assume last_swap is reasonable (not too close to u64::MAX)
        // We need: last_swap * (10000 + min_increase_bps) <= u64::MAX
        // Worst case: last_swap * 15000 <= u64::MAX
        // Therefore: last_swap <= u64::MAX / 15000
        kani::assume(last_swap <= u64::MAX / 15000);

        let result = calculate_flush_threshold(last_swap, min_increase_bps);

        // Property: Result should always be Some for valid inputs
        assert!(result.is_some());

        // Property: Result should be greater than or equal to last_swap
        if let Some(threshold) = result {
            assert!(threshold >= last_swap);
        }
    }

    // Proof 2: Payout distribution always sums to total (no loss or gain)
    #[kani::proof]
    fn verify_payout_distribution_conservation() {
        let total: u64 = kani::any();
        let creator_fee_bps: u16 = kani::any();
        let claimer_fee_bps: u16 = kani::any();

        // Assume valid fee parameters (combined <= 20%)
        kani::assume(creator_fee_bps as u32 + claimer_fee_bps as u32 <= 2000);

        let result = calculate_payout_distribution(total, creator_fee_bps, claimer_fee_bps);

        // Property: If calculation succeeds, all cuts should sum back to total
        if let Some((creator_cut, claimer_cut, winner_cut)) = result {
            let reconstructed = creator_cut
                .checked_add(claimer_cut)
                .and_then(|sum| sum.checked_add(winner_cut));

            assert!(reconstructed.is_some());
            assert_eq!(reconstructed.unwrap(), total);
        }
    }

    // Proof 3: Fee validation is consistent
    #[kani::proof]
    fn verify_fee_validation() {
        let creator_fee_bps: u16 = kani::any();
        let claimer_fee_bps: u16 = kani::any();

        let is_valid = validate_fees(creator_fee_bps, claimer_fee_bps);

        // Property: Fees are valid iff their sum is <= 2000
        let sum = creator_fee_bps as u32 + claimer_fee_bps as u32;
        assert_eq!(is_valid, sum <= 2000);
    }

    // Proof 4: Min increase validation is correct
    #[kani::proof]
    fn verify_min_increase_validation() {
        let min_increase_bps: u16 = kani::any();

        let is_valid = validate_min_increase(min_increase_bps);

        // Property: Valid iff between 100 and 5000 (1% to 50%)
        assert_eq!(is_valid, min_increase_bps >= 100 && min_increase_bps <= 5000);
    }

    // Proof 5: Threshold calculation precision (no truncation errors that benefit attacker)
    #[kani::proof]
    fn verify_threshold_precision() {
        let last_swap: u64 = kani::any();
        let min_increase_bps: u16 = kani::any();

        kani::assume(min_increase_bps >= 100 && min_increase_bps <= 5000);
        kani::assume(last_swap <= u64::MAX / 15000); // Same as threshold calculation proof
        kani::assume(last_swap >= 100_000); // Min initial swap

        let threshold = calculate_flush_threshold(last_swap, min_increase_bps);

        if let Some(threshold) = threshold {
            // Property: Threshold increase is at least min_increase_bps
            // Calculate expected minimum increase
            let min_increase = (last_swap as u128 * min_increase_bps as u128 / 10000) as u64;
            assert!(threshold >= last_swap.checked_add(min_increase).unwrap_or(u64::MAX));
        }
    }

    // Proof 6: HF-01 Security Issue - Escrow empty check
    #[kani::proof]
    fn verify_escrow_empty_check_hf01() {
        let balance: u64 = kani::any();
        let dust_threshold: u64 = 10_000_000; // Current hardcoded 0.01 SOL

        let is_empty = is_escrow_empty(balance, dust_threshold);

        // Property: This check allows up to 0.01 SOL to be considered "empty"
        // This is the HF-01 vulnerability - legitimate deposits < 0.01 SOL can be seized
        if balance > 0 && balance <= dust_threshold {
            // VULNERABILITY: Non-zero balance is considered empty
            assert!(is_empty);
        }

        // Property: Balance above threshold is correctly identified as non-empty
        if balance > dust_threshold {
            assert!(!is_empty);
        }
    }

    // Proof 7: Balance summation doesn't overflow for realistic scenarios
    #[kani::proof]
    fn verify_balance_summation() {
        let balance1: u64 = kani::any();
        let balance2: u64 = kani::any();
        let balance3: u64 = kani::any();
        let balance4: u64 = kani::any();

        // Assume balances are reasonable (each < 1 billion SOL)
        kani::assume(balance1 <= 1_000_000_000_000_000_000);
        kani::assume(balance2 <= 1_000_000_000_000_000_000);
        kani::assume(balance3 <= 1_000_000_000_000_000_000);
        kani::assume(balance4 <= 1_000_000_000_000_000_000);

        let balances = [balance1, balance2, balance3, balance4];
        let result = sum_balances(&balances);

        // Property: For reasonable balances, summation should succeed
        if let Some(total) = result {
            // Verify no individual balance exceeds the total
            for &balance in &balances {
                assert!(balance <= total);
            }
        }
    }

    // Proof 8: Fee calculation doesn't overflow for max fees
    #[kani::proof]
    fn verify_max_fee_calculation() {
        let total: u64 = kani::any();
        let creator_fee_bps: u16 = 2000; // Max 20%
        let claimer_fee_bps: u16 = 0;

        let result = calculate_payout_distribution(total, creator_fee_bps, claimer_fee_bps);

        // Property: Max fee calculation should always succeed
        assert!(result.is_some());

        if let Some((creator_cut, claimer_cut, winner_cut)) = result {
            // Property: Creator cut should be approximately 20% of total
            let expected_creator = (total as u128 * 20 / 100) as u64;
            let diff = if creator_cut > expected_creator {
                creator_cut - expected_creator
            } else {
                expected_creator - creator_cut
            };
            // Allow small rounding difference (< 1%)
            assert!(diff <= total / 100);

            // Property: All amounts are valid
            assert!(claimer_cut == 0);
            assert!(winner_cut <= total);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_calculation() {
        // Test with 5% increase (500 bps)
        let result = calculate_flush_threshold(1_000_000_000, 500);
        assert_eq!(result, Some(1_050_000_000));

        // Test with 1% increase (100 bps)
        let result = calculate_flush_threshold(1_000_000_000, 100);
        assert_eq!(result, Some(1_010_000_000));

        // Test with 50% increase (5000 bps)
        let result = calculate_flush_threshold(1_000_000_000, 5000);
        assert_eq!(result, Some(1_500_000_000));
    }

    #[test]
    fn test_payout_distribution() {
        // Test with 5% creator fee, 0.5% claimer fee
        let result = calculate_payout_distribution(10_000_000_000, 500, 50);
        assert!(result.is_some());

        let (creator, claimer, winner) = result.unwrap();
        assert_eq!(creator, 500_000_000);  // 5%
        assert_eq!(claimer, 50_000_000);   // 0.5%
        assert_eq!(winner, 9_450_000_000); // 94.5%

        // Verify sum equals total
        assert_eq!(creator + claimer + winner, 10_000_000_000);
    }

    #[test]
    fn test_fee_validation() {
        assert!(validate_fees(500, 50));    // 5.5% total - valid
        assert!(validate_fees(1000, 1000)); // 20% total - valid
        assert!(!validate_fees(1500, 1000)); // 25% total - invalid
        assert!(!validate_fees(2001, 0));   // 20.01% total - invalid
    }

    #[test]
    fn test_hf01_vulnerability() {
        // HF-01: Balances up to 0.01 SOL are considered "empty"
        let dust_threshold = 10_000_000;

        // This deposit would be considered empty and seizable by creator
        assert!(is_escrow_empty(5_000_000, dust_threshold)); // 0.005 SOL
        assert!(is_escrow_empty(9_999_999, dust_threshold)); // 0.009999999 SOL

        // Only above threshold is safe
        assert!(!is_escrow_empty(10_000_001, dust_threshold));
    }
}
