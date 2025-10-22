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

/// ClaimPayout instruction has no additional data
pub fn process_claim_payout(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [bucket_account, main_bucket, escrow_a, escrow_b, creator, claimer, winner] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify bucket is owned by program
    if bucket_account.owner() != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    // Load bucket state
    let bucket = Bucket::from_account_info_unchecked(bucket_account)?;

    // Verify signer is claimer
    if !claimer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
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

    // Verify creator address
    if creator.key() != &bucket.creator_address {
        return Err(ProgramError::InvalidAccountData);
    }

    // Verify winner address
    if winner.key() != &bucket.current_target {
        return Err(ProgramError::InvalidAccountData);
    }

    // Get current epoch
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // Verify 3 epochs have passed since last flip
    if current_epoch < bucket.last_flip_epoch + 3 {
        return Err(HateFunError::ClaimTooEarly.into());
    }

    // Calculate total balance
    let main_balance = main_bucket.lamports();
    let escrow_a_balance = escrow_a.lamports();
    let escrow_b_balance = escrow_b.lamports();
    let bucket_balance = bucket_account.lamports();

    let total = main_balance
        .checked_add(escrow_a_balance)
        .ok_or(HateFunError::Overflow)?
        .checked_add(escrow_b_balance)
        .ok_or(HateFunError::Overflow)?
        .checked_add(bucket_balance)
        .ok_or(HateFunError::Overflow)?;

    // Calculate fee distributions
    let creator_cut = (total as u128 * bucket.creator_fee_bps as u128 / 10000) as u64;
    let claimer_cut = (total as u128 * bucket.claimer_fee_bps as u128 / 10000) as u64;
    let winner_cut = total
        .checked_sub(creator_cut)
        .ok_or(HateFunError::Overflow)?
        .checked_sub(claimer_cut)
        .ok_or(HateFunError::Overflow)?;

    // Transfer funds
    // First, collect all funds to bucket account
    unsafe {
        *bucket_account.borrow_mut_lamports_unchecked() += main_balance + escrow_a_balance + escrow_b_balance;
        *main_bucket.borrow_mut_lamports_unchecked() = 0;
        *escrow_a.borrow_mut_lamports_unchecked() = 0;
        *escrow_b.borrow_mut_lamports_unchecked() = 0;
    }

    // Now distribute from bucket account
    unsafe {
        *bucket_account.borrow_mut_lamports_unchecked() -= creator_cut;
        *creator.borrow_mut_lamports_unchecked() += creator_cut;

        *bucket_account.borrow_mut_lamports_unchecked() -= claimer_cut;
        *claimer.borrow_mut_lamports_unchecked() += claimer_cut;

        *bucket_account.borrow_mut_lamports_unchecked() -= winner_cut;
        *winner.borrow_mut_lamports_unchecked() += winner_cut;
    }

    // Close all PDAs by setting their lamports to 0 and data length to 0
    // (bucket_account lamports should now be 0 or very close to 0)

    Ok(())
}
