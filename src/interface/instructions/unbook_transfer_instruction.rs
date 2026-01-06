use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

#[repr(C)]
pub struct UnbookTransferInstructionData {
    pub discriminator: u8,
    vault_index: [u8; size_of::<u64>()],
    transfer_index: [u8; size_of::<u64>()],
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for UnbookTransferInstructionData {}

impl UnbookTransferInstructionData {
    pub fn new(vault_index: u64, transfer_index: u64) -> Self{
        Self { 
            discriminator: PimeInstruction::UnbookTransfer as u8, 
            vault_index: vault_index.to_le_bytes(), 
            transfer_index: transfer_index.to_le_bytes(),
        }
    }

    pub fn vault_index(&self) -> u64 {
        u64::from_le_bytes(self.vault_index)
    }

    pub fn transfer_index(&self) -> u64 {
        u64::from_le_bytes(self.transfer_index)
    }
}

