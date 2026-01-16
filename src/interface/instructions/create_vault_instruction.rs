use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

#[repr(C)]
pub struct CreateVaultInstructionData {
    pub discriminator: u8,
    vault_index: [u8; size_of::<u64>()],
    timeframe: [u8; size_of::<i64>()],
    max_transactions: [u8; size_of::<u64>()],
    max_amount: [u8; size_of::<u64>()],
    allows_transfers: u8,
    transfer_min_warmup: [u8; size_of::<u64>()],
    transfer_max_window: [u8; size_of::<u64>()],
}

impl CreateVaultInstructionData {
    
    pub fn new(vault_index: u64, timeframe: i64, max_transactions: u64, max_amount: u64, allows_transfers: bool, transfer_min_warmup: u64, transfer_max_window: u64,) -> Self{
        Self { 
            discriminator: PimeInstruction::CreateVault as u8, 
            vault_index: vault_index.to_le_bytes(), 
            timeframe: timeframe.to_le_bytes(), 
            max_transactions: max_transactions.to_le_bytes(),
            max_amount: max_amount.to_le_bytes(), 
            allows_transfers: if allows_transfers { 1u8 } else { 0u8 },
            transfer_min_warmup: transfer_min_warmup.to_le_bytes(),
            transfer_max_window: transfer_max_window.to_le_bytes(),
        }
    }

    pub fn vault_index(&self) -> u64 {
        u64::from_le_bytes(self.vault_index)
    }

    pub fn timeframe(&self) -> i64 {
        i64::from_le_bytes(self.timeframe)
    }

    pub fn max_transactions(&self) -> u64 {
        u64::from_le_bytes(self.max_transactions)
    }

    pub fn max_amount(&self) -> u64 {
        u64::from_le_bytes(self.max_amount)
    }
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for CreateVaultInstructionData {
    const LEN: usize = size_of::<Self>();
}
