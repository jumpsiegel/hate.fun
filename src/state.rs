use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// The main Bucket account that stores all parameters and state
#[repr(C)]
pub struct Bucket {
    pub address_a: Pubkey,           // 32 bytes - First competing address
    pub address_b: Pubkey,           // 32 bytes - Second competing address
    pub creator_address: Pubkey,     // 32 bytes - Receives creator fee
    pub current_target: Pubkey,      // 32 bytes - Current winner (A or B)
    pub last_swap: u64,              // 8 bytes - Amount needed to be exceeded
    pub creation_epoch: u64,         // 8 bytes - Epoch when bucket was created
    pub last_flip_epoch: u64,        // 8 bytes - Last epoch when target flipped
    pub creator_fee_bps: u16,        // 2 bytes - Creator fee in basis points
    pub claimer_fee_bps: u16,        // 2 bytes - Claimer fee in basis points
    pub min_increase_bps: u16,       // 2 bytes - Minimum increase percentage
    pub bump: u8,                    // 1 byte - PDA bump seed
}

impl Bucket {
    /// Size of Bucket account in bytes
    pub const SIZE: usize = 32 + 32 + 32 + 32 + 8 + 8 + 8 + 2 + 2 + 2 + 1;

    /// Seed prefix for Bucket PDA
    pub const SEED_PREFIX: &'static [u8] = b"bucket";

    /// Seed prefix for main bucket PDA
    pub const MAIN_SEED_PREFIX: &'static [u8] = b"main";

    /// Seed prefix for escrow A PDA
    pub const ESCROW_A_SEED_PREFIX: &'static [u8] = b"escrow_a";

    /// Seed prefix for escrow B PDA
    pub const ESCROW_B_SEED_PREFIX: &'static [u8] = b"escrow_b";

    /// Deserialize a Bucket from account data
    pub fn from_account_info(account: &AccountInfo) -> Result<&mut Self, ProgramError> {
        let data = unsafe { &mut *account.borrow_mut_data_unchecked().as_mut_ptr().cast::<Self>() };
        Ok(data)
    }

    /// Deserialize a Bucket from account data (immutable)
    pub fn from_account_info_unchecked(account: &AccountInfo) -> Result<&Self, ProgramError> {
        let data = unsafe { &*account.borrow_data_unchecked().as_ptr().cast::<Self>() };
        Ok(data)
    }
}

/// PDA derivation helpers
pub mod pda {
    use super::*;
    use pinocchio::pubkey::find_program_address;

    /// Derive bucket PDA address
    pub fn derive_bucket_address(
        creator: &Pubkey,
        seed: &[u8],
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        find_program_address(
            &[Bucket::SEED_PREFIX, creator.as_ref(), seed],
            program_id,
        )
    }

    /// Derive main bucket PDA address
    pub fn derive_main_bucket_address(
        bucket: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        find_program_address(
            &[Bucket::MAIN_SEED_PREFIX, bucket.as_ref()],
            program_id,
        )
    }

    /// Derive escrow A PDA address
    pub fn derive_escrow_a_address(
        bucket: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        find_program_address(
            &[Bucket::ESCROW_A_SEED_PREFIX, bucket.as_ref()],
            program_id,
        )
    }

    /// Derive escrow B PDA address
    pub fn derive_escrow_b_address(
        bucket: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        find_program_address(
            &[Bucket::ESCROW_B_SEED_PREFIX, bucket.as_ref()],
            program_id,
        )
    }
}
