use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::HateFunError,
    state::{Bucket, pda},
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
    // Escrows are PDAs that need ~890,880 lamports for rent exemption
    // We consider them "empty" if they have less than 0.01 SOL (10,000,000 lamports)
    // This accounts for rent-exempt minimum plus any dust
    const ESCROW_EMPTY_THRESHOLD: u64 = 10_000_000; // 0.01 SOL

    let escrow_a_balance = escrow_a.lamports();
    let escrow_b_balance = escrow_b.lamports();

    if escrow_a_balance > ESCROW_EMPTY_THRESHOLD || escrow_b_balance > ESCROW_EMPTY_THRESHOLD {
        return Err(HateFunError::EscrowsNotEmpty.into());
    }

    // Calculate total to return (all PDA balances including rent)
    let main_balance = main_bucket.lamports();
    let bucket_balance = bucket_account.lamports();

    let total = main_balance
        .checked_add(bucket_balance)
        .and_then(|sum| sum.checked_add(escrow_a_balance))
        .and_then(|sum| sum.checked_add(escrow_b_balance))
        .ok_or(HateFunError::Overflow)?;

    // Transfer all funds to creator
    unsafe {
        *bucket_account.borrow_mut_lamports_unchecked() = 0;
        *main_bucket.borrow_mut_lamports_unchecked() = 0;
        *escrow_a.borrow_mut_lamports_unchecked() = 0;
        *escrow_b.borrow_mut_lamports_unchecked() = 0;
        *creator.borrow_mut_lamports_unchecked() += total;
    }

    Ok(())
}
