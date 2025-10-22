use pinocchio::{
    account_info::AccountInfo,
    cpi::{invoke, invoke_signed},
    instruction::{AccountMeta, Instruction, Seed, Signer},
    pubkey::Pubkey,
    ProgramResult,
};

/// System Program ID
pub const ID: Pubkey = [
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// System Program instruction discriminators
const CREATE_ACCOUNT: u32 = 0;
const TRANSFER: u32 = 2;

/// Create a new account
pub fn create_account<'a>(
    from: &'a AccountInfo,
    to: &'a AccountInfo,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
    seeds: &[Seed],
) -> ProgramResult {
    let mut instruction_data = [0u8; 52];
    // discriminator (4 bytes)
    instruction_data[0..4].copy_from_slice(&CREATE_ACCOUNT.to_le_bytes());
    // lamports (8 bytes)
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
    // space (8 bytes)
    instruction_data[12..20].copy_from_slice(&space.to_le_bytes());
    // owner (32 bytes)
    instruction_data[20..52].copy_from_slice(owner.as_ref());

    let accounts = [
        AccountMeta::writable_signer(from.key()),
        AccountMeta::writable_signer(to.key()),
    ];

    let instruction = Instruction {
        program_id: &ID,
        data: &instruction_data,
        accounts: &accounts,
    };

    let account_infos = [from, to];
    let signers = [Signer::from(seeds)];

    invoke_signed(&instruction, &account_infos, &signers)
}

/// Transfer lamports
pub fn transfer<'a>(
    from: &'a AccountInfo,
    to: &'a AccountInfo,
    lamports: u64,
) -> ProgramResult {
    let mut instruction_data = [0u8; 12];
    // discriminator (4 bytes)
    instruction_data[0..4].copy_from_slice(&TRANSFER.to_le_bytes());
    // lamports (8 bytes)
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());

    let accounts = [
        AccountMeta::writable_signer(from.key()),
        AccountMeta::writable(to.key()),
    ];

    let instruction = Instruction {
        program_id: &ID,
        data: &instruction_data,
        accounts: &accounts,
    };

    let account_infos = [from, to];

    invoke(&instruction, &account_infos)
}
