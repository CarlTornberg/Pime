use pinocchio::pubkey::Pubkey;

use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

#[repr(C)]
pub struct DepositToVaultInstructionData {
    pub discriminator: u8,
    vault_owner: Pubkey,
    index: [u8; size_of::<u64>()],
    amount: [u8; size_of::<u64>()],
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for DepositToVaultInstructionData {}

impl DepositToVaultInstructionData {
    
    pub fn new(vault_owner: Pubkey, index: u64, amount: u64) -> Self{
        Self { 
            discriminator: PimeInstruction::DepositToVault as u8, 
            vault_owner, 
            index: index.to_le_bytes(), 
            amount: amount.to_le_bytes(),
        }
    }

    pub fn vault_owner(&self) -> Pubkey {
        self.vault_owner
    }

    pub fn index(&self) -> u64 {
        u64::from_le_bytes(self.index)
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

}

