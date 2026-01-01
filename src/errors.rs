use pinocchio::program_error::ProgramError;

#[repr(u8)]
pub enum PimeError {
    IncorrectPDA = 0,
    InvalidTokenProgram = 1,
    InvalidMintTokenProgram = 2,
    Unserializeable = 3,
    Undeserializeable = 4,
    WithdrawLimitReachedTransactions,
    WithdrawLimitReachedAmount,
    AuthorityError,
    UnsupporedTokenProgram,
    VaultWarmupViolation,
    DestinationMismatch,



    Unknown = u8::MAX,
}

impl From<u8> for PimeError {
    fn from(value: u8) -> Self {
        match value { 
            // SAFETY value is guaranteed to be in range of PimeError enum
            0..=0 => unsafe { core::mem::transmute::<u8, PimeError>(value) },
            _ => PimeError::Unknown
        }
    }
}

impl From<PimeError> for ProgramError {
    fn from(value: PimeError) -> Self {
        ProgramError::Custom(value as u32)
    }
}
