use pinocchio::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum HateFunError {
    /// Combined fees exceed maximum of 20%
    FeesTooHigh = 0,
    /// Creator address must be different from address A and B
    CreatorMustBeDifferent = 1,
    /// Minimum increase must be between 1% and 50%
    InvalidMinimumIncrease = 2,
    /// Initial last swap must be at least 0.0001 SOL
    InitialSwapTooLow = 3,
    /// Escrow balance is below required threshold
    InsufficientEscrowBalance = 4,
    /// Cannot claim payout before 3 epochs have passed
    ClaimTooEarly = 5,
    /// Only creator can close the bucket
    UnauthorizedClose = 6,
    /// Cannot close bucket after first flip
    BucketHasFlips = 7,
    /// Cannot close bucket with non-empty escrows
    EscrowsNotEmpty = 8,
    /// Invalid escrow account provided
    InvalidEscrow = 9,
    /// Arithmetic overflow
    Overflow = 10,
    /// Deposit amount is too small (below minimum)
    DepositTooSmall = 11,
    /// Zero amount deposit not allowed
    ZeroAmountDeposit = 12,
}

impl From<HateFunError> for ProgramError {
    fn from(e: HateFunError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
