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
    verification::{calculate_payout_distribution, sum_balances},
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

    // Calculate total balance using VERIFIED function
    let balances = [
        main_bucket.lamports(),
        escrow_a.lamports(),
        escrow_b.lamports(),
        bucket_account.lamports(),
    ];
    let total = sum_balances(&balances)
        .ok_or(HateFunError::Overflow)?;

    // Calculate fee distributions using VERIFIED function
    // Kani proved this conserves value: creator_cut + claimer_cut + winner_cut = total
    let (creator_cut, claimer_cut, winner_cut) = calculate_payout_distribution(
        total,
        bucket.creator_fee_bps,
        bucket.claimer_fee_bps,
    ).ok_or(HateFunError::Overflow)?;

    // Transfer funds
    // First, collect all funds to bucket account
    unsafe {
        *bucket_account.borrow_mut_lamports_unchecked() += balances[0] + balances[1] + balances[2];
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
