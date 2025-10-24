use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{error::HateFunError, system_program};
use super::read_u64;

/// DepositToEscrow instruction data layout:
/// [0..8] amount: u64
pub fn process_deposit_to_escrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = read_u64(data, 0)?;

    // Validate deposit amount
    // Prevent zero deposits (standardized to use HateFunError)
    if amount == 0 {
        return Err(HateFunError::ZeroAmountDeposit.into());
    }

    // Prevent dust deposits that could complicate bucket closure
    // Minimum 0.000001 SOL (1,000 lamports) to prevent griefing
    const MINIMUM_DEPOSIT: u64 = 1_000; // 0.000001 SOL
    if amount < MINIMUM_DEPOSIT {
        return Err(HateFunError::DepositTooSmall.into());
    }

    // Parse accounts
    let [depositor, target_escrow, _system_program_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signer
    if !depositor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify escrow is owned by program
    if target_escrow.owner() != program_id {
        return Err(HateFunError::InvalidEscrow.into());
    }

    // Transfer lamports from depositor to escrow
    system_program::transfer(depositor, target_escrow, amount)?;

    Ok(())
}
