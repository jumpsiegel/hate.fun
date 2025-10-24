use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{
    error::HateFunError,
    state::{Bucket, pda},
    verification::sum_balances,
};

/// CloseBucket instruction has no additional data
pub fn process_close_bucket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [creator, bucket_account, main_bucket, escrow_a, escrow_b] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify bucket is owned by program
    if bucket_account.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load bucket state
    let bucket = Bucket::from_account_info_unchecked(bucket_account)?;

    // Verify signer is creator
    if !creator.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if creator.key() != &bucket.creator_address {
        return Err(HateFunError::UnauthorizedClose.into());
    }

    // Verify no flips have occurred (last_flip_epoch == creation_epoch)
    if bucket.last_flip_epoch != bucket.creation_epoch {
        return Err(HateFunError::BucketHasFlips.into());
    }

    // Verify PDAs
    let (main_bucket_pda, _) = pda::derive_main_bucket_address(bucket_account.key(), program_id);
    if main_bucket.key() != &main_bucket_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let (escrow_a_pda, _) = pda::derive_escrow_a_address(bucket_account.key(), program_id);
    if escrow_a.key() != &escrow_a_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let (escrow_b_pda, _) = pda::derive_escrow_b_address(bucket_account.key(), program_id);
    if escrow_b.key() != &escrow_b_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Verify escrows are empty (only contain rent-exempt balance, no user deposits)
    // FIX HF-01: Use actual rent-exempt minimum instead of arbitrary 0.01 SOL threshold
    // This prevents creators from seizing legitimate deposits below the old threshold
    let rent = Rent::get()?;
    let rent_exempt_minimum = rent.minimum_balance(0); // ~890,880 lamports for empty account

    let escrow_a_balance = escrow_a.lamports();
    let escrow_b_balance = escrow_b.lamports();

    // Allow closure only if escrows contain exactly rent-exempt amount (no user deposits)
    if escrow_a_balance > rent_exempt_minimum || escrow_b_balance > rent_exempt_minimum {
        return Err(HateFunError::EscrowsNotEmpty.into());
    }

    // Calculate total to return (all PDA balances including rent)
    // Use verified sum_balances function to prevent overflow
    let balances = [
        main_bucket.lamports(),
        bucket_account.lamports(),
        escrow_a_balance,
        escrow_b_balance,
    ];
    let total = sum_balances(&balances).ok_or(HateFunError::Overflow)?;

    // Transfer all funds to creator
    // SAFETY: These unsafe operations are justified because:
    // 1. We've verified all account ownership and PDAs above
    // 2. We've calculated total using verified sum_balances (no overflow)
    // 3. The transaction is atomic - either all transfers succeed or none do
    // 4. We zero out source accounts before crediting destination to prevent double-spend
    unsafe {
        *bucket_account.borrow_mut_lamports_unchecked() = 0;
        *main_bucket.borrow_mut_lamports_unchecked() = 0;
        *escrow_a.borrow_mut_lamports_unchecked() = 0;
        *escrow_b.borrow_mut_lamports_unchecked() = 0;
        *creator.borrow_mut_lamports_unchecked() += total;
    }

    Ok(())
}
