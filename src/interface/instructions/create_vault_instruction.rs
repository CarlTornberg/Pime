use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

#[repr(C)]
pub struct CreateVaultInstructionData {
    pub discriminator: u8,
    index: [u8; size_of::<u64>()],
    timeframe: [u8; size_of::<i64>()],
    max_transactions: [u8; size_of::<u64>()],
    max_lamports: [u8; size_of::<u64>()],
}

impl CreateVaultInstructionData {
    
    pub fn new(index: u64, timeframe: i64, max_transactions: u64, max_lamports: u64) -> Self{
        Self { 
            discriminator: PimeInstruction::CreateVault as u8, 
            index: index.to_le_bytes(), 
            timeframe: timeframe.to_le_bytes(), 
            max_transactions: max_transactions.to_le_bytes(),
            max_lamports: max_lamports.to_le_bytes(), 
        }
    }

    pub fn index(&self) -> u64 {
        u64::from_le_bytes(self.index)
    }

    pub fn timeframe(&self) -> i64 {
        i64::from_le_bytes(self.timeframe)
    }

    pub fn max_transactions(&self) -> u64 {
        u64::from_le_bytes(self.max_transactions)
    }

    pub fn max_lamports(&self) -> u64 {
        u64::from_le_bytes(self.max_lamports)
    }
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for CreateVaultInstructionData {}
