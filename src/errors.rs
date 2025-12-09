use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

#[repr(u8)]
pub enum PimeError {
    IncorrectPDA(Pubkey, Pubkey) = 0,
}


