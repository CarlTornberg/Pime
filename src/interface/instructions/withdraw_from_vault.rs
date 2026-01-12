use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

#[repr(C)]
pub struct WithdrawFromVaultInstructionData {
    pub discriminator: u8,
    vault_index: [u8; size_of::<u64>()],
    amount: [u8; size_of::<u64>()],
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for WithdrawFromVaultInstructionData {
    const LEN: usize = size_of::<Self>();
}

impl WithdrawFromVaultInstructionData {
    
    pub fn new(amount: u64, vault_index: u64) -> Self{
        Self { 
            discriminator: PimeInstruction::WithdrawFromVault as u8, 
            vault_index: vault_index.to_le_bytes(),
            amount: amount.to_le_bytes(),
        }
    }

    pub fn vault_index(&self) -> u64 {
        u64::from_le_bytes(self.vault_index)
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }
}

