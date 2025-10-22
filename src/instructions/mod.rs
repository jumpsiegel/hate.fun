use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

pub mod create_bucket;
pub mod deposit_to_escrow;
pub mod flush_escrow;
pub mod claim_payout;
pub mod close_bucket;

use create_bucket::process_create_bucket;
use deposit_to_escrow::process_deposit_to_escrow;
use flush_escrow::process_flush_escrow;
use claim_payout::process_claim_payout;
use close_bucket::process_close_bucket;

/// Instruction discriminators
#[repr(u8)]
pub enum GateInstruction {
    CreateBucket = 0,
    DepositToEscrow = 1,
    FlushEscrow = 2,
    ClaimPayout = 3,
    CloseBucket = 4,
}

/// Main instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    match instruction_data[0] {
        0 => process_create_bucket(program_id, accounts, &instruction_data[1..]),
        1 => process_deposit_to_escrow(program_id, accounts, &instruction_data[1..]),
        2 => process_flush_escrow(program_id, accounts, &instruction_data[1..]),
        3 => process_claim_payout(program_id, accounts, &instruction_data[1..]),
        4 => process_close_bucket(program_id, accounts, &instruction_data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/// Helper function to read u64 from little-endian bytes
pub fn read_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    if data.len() < offset + 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[offset..offset + 8]);
    Ok(u64::from_le_bytes(bytes))
}

/// Helper function to read u16 from little-endian bytes
pub fn read_u16(data: &[u8], offset: usize) -> Result<u16, ProgramError> {
    if data.len() < offset + 2 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut bytes = [0u8; 2];
    bytes.copy_from_slice(&data[offset..offset + 2]);
    Ok(u16::from_le_bytes(bytes))
}

/// Helper function to read Pubkey from bytes
pub fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey, ProgramError> {
    if data.len() < offset + 32 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&data[offset..offset + 32]);
    Ok(Pubkey::from(bytes))
}
