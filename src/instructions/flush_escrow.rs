use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    error::HateFunError,
    state::{Bucket, pda},
};

/// FlushEscrow instruction has no additional data
pub fn process_flush_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [bucket_account, main_bucket, escrow_to_flush] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify bucket is owned by program
    if bucket_account.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load bucket state
    let bucket = Bucket::from_account_info(bucket_account)?;

    // Verify escrow belongs to this bucket
    let (escrow_a_pda, _) = pda::derive_escrow_a_address(bucket_account.key(), program_id);
    let (escrow_b_pda, _) = pda::derive_escrow_b_address(bucket_account.key(), program_id);

    let is_escrow_a = escrow_to_flush.key() == &escrow_a_pda;
    let is_escrow_b = escrow_to_flush.key() == &escrow_b_pda;

    if !is_escrow_a && !is_escrow_b {
        return Err(HateFunError::InvalidEscrow.into());
    }

    // Verify main bucket PDA
    let (main_bucket_pda, _) = pda::derive_main_bucket_address(bucket_account.key(), program_id);
    if main_bucket.key() != &main_bucket_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Get escrow balance
    let escrow_balance = escrow_to_flush.lamports();

    // Calculate required threshold: last_swap * (10000 + min_increase_bps) / 10000
    let threshold = bucket.last_swap
        .checked_mul(10000 + bucket.min_increase_bps as u64)
        .ok_or(HateFunError::Overflow)?
        .checked_div(10000)
        .ok_or(HateFunError::Overflow)?;

    // Verify escrow balance meets threshold
    if escrow_balance < threshold {
        return Err(HateFunError::InsufficientEscrowBalance.into());
    }

    // Get current epoch
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // Transfer entire escrow balance to main bucket
    unsafe {
        *escrow_to_flush.borrow_mut_lamports_unchecked() = 0;
        *main_bucket.borrow_mut_lamports_unchecked() += escrow_balance;
    }

    // Flip current target
    let new_target = if bucket.current_target == bucket.address_a {
        bucket.address_b
    } else {
        bucket.address_a
    };

    // Update bucket state
    bucket.current_target = new_target;
    bucket.last_swap = escrow_balance;
    bucket.last_flip_epoch = current_epoch;

    Ok(())
}
