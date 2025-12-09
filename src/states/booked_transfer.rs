use pinocchio::{program_error::ProgramError, pubkey::{Pubkey, find_program_address}};

#[repr(C)]
pub struct BookedTransfer {
    pub discriminator: u8,
    pub bump: u8,
    pub authority: Pubkey,
    pub original_vault: Pubkey,
}

impl BookedTransfer {
}
