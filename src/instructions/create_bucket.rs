use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio::instruction::Seed;

use crate::{
    error::HateFunError,
    state::{Bucket, pda},
    system_program,
};
use super::{read_u64, read_u16, read_pubkey};

/// CreateBucket instruction data layout:
/// [0..32]   address_a: Pubkey
/// [32..64]  address_b: Pubkey
/// [64..96]  creator_address: Pubkey
/// [96..98]  creator_fee_bps: u16
/// [98..100] claimer_fee_bps: u16
/// [100..108] initial_last_swap: u64
/// [108..110] min_increase_bps: u16
/// [110..142] seed: [u8; 32]
pub fn process_create_bucket(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    if data.len() < 142 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let address_a = read_pubkey(data, 0)?;
    let address_b = read_pubkey(data, 32)?;
    let creator_address = read_pubkey(data, 64)?;
    let creator_fee_bps = read_u16(data, 96)?;
    let claimer_fee_bps = read_u16(data, 98)?;
    let initial_last_swap = read_u64(data, 100)?;
    let min_increase_bps = read_u16(data, 108)?;

    let mut seed_bytes = [0u8; 32];
    seed_bytes.copy_from_slice(&data[110..142]);

    // Validate parameters
    if creator_fee_bps as u32 + claimer_fee_bps as u32 > 2000 {
        return Err(HateFunError::FeesTooHigh.into());
    }

    if creator_address == address_a || creator_address == address_b {
        return Err(HateFunError::CreatorMustBeDifferent.into());
    }

    if min_increase_bps < 100 || min_increase_bps > 5000 {
        return Err(HateFunError::InvalidMinimumIncrease.into());
    }

    if initial_last_swap < 100_000 {
        return Err(HateFunError::InitialSwapTooLow.into());
    }

    // Parse accounts
    let [payer, bucket_account, main_bucket, escrow_a, escrow_b, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive PDAs
    let (bucket_pda, bucket_bump) = pda::derive_bucket_address(
        &creator_address,
        &seed_bytes,
        program_id,
    );

    if bucket_account.key() != &bucket_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let (main_bucket_pda, main_bump) = pda::derive_main_bucket_address(bucket_account.key(), program_id);
    if main_bucket.key() != &main_bucket_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let (escrow_a_pda, escrow_a_bump) = pda::derive_escrow_a_address(bucket_account.key(), program_id);
    if escrow_a.key() != &escrow_a_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let (escrow_b_pda, escrow_b_bump) = pda::derive_escrow_b_address(bucket_account.key(), program_id);
    if escrow_b.key() != &escrow_b_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Get current epoch
    let clock = Clock::get()?;
    let current_epoch = clock.epoch;

    // Calculate rent exemption
    let rent = Rent::get()?;
    let bucket_rent = rent.minimum_balance(Bucket::SIZE);
    let escrow_rent = rent.minimum_balance(0); // Empty accounts

    // Create bucket account
    let bucket_bump_arr = [bucket_bump];
    let bucket_seeds = [
        Seed::from(Bucket::SEED_PREFIX),
        Seed::from(creator_address.as_ref()),
        Seed::from(seed_bytes.as_ref()),
        Seed::from(&bucket_bump_arr),
    ];

    system_program::create_account(
        payer,
        bucket_account,
        bucket_rent,
        Bucket::SIZE as u64,
        program_id,
        &bucket_seeds,
    )?;

    // Create main bucket PDA
    let main_bump_arr = [main_bump];
    let main_seeds = [
        Seed::from(Bucket::MAIN_SEED_PREFIX),
        Seed::from(bucket_account.key().as_ref()),
        Seed::from(&main_bump_arr),
    ];

    system_program::create_account(
        payer,
        main_bucket,
        escrow_rent,
        0,
        program_id,
        &main_seeds,
    )?;

    // Create escrow A PDA
    let escrow_a_bump_arr = [escrow_a_bump];
    let escrow_a_seeds = [
        Seed::from(Bucket::ESCROW_A_SEED_PREFIX),
        Seed::from(bucket_account.key().as_ref()),
        Seed::from(&escrow_a_bump_arr),
    ];

    system_program::create_account(
        payer,
        escrow_a,
        escrow_rent,
        0,
        program_id,
        &escrow_a_seeds,
    )?;

    // Create escrow B PDA
    let escrow_b_bump_arr = [escrow_b_bump];
    let escrow_b_seeds = [
        Seed::from(Bucket::ESCROW_B_SEED_PREFIX),
        Seed::from(bucket_account.key().as_ref()),
        Seed::from(&escrow_b_bump_arr),
    ];

    system_program::create_account(
        payer,
        escrow_b,
        escrow_rent,
        0,
        program_id,
        &escrow_b_seeds,
    )?;

    // Initialize bucket state
    let bucket = Bucket::from_account_info(bucket_account)?;
    bucket.address_a = address_a;
    bucket.address_b = address_b;
    bucket.creator_address = creator_address;
    bucket.current_target = address_a; // Start pointing at address A
    bucket.last_swap = initial_last_swap;
    bucket.creation_epoch = current_epoch;
    bucket.last_flip_epoch = current_epoch;
    bucket.creator_fee_bps = creator_fee_bps;
    bucket.claimer_fee_bps = claimer_fee_bps;
    bucket.min_increase_bps = min_increase_bps;
    bucket.bump = bucket_bump;

    Ok(())
}
