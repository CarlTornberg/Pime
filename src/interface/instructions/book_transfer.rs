use pinocchio::sysvars::clock::UnixTimestamp;

use crate::{interface::pime_instruction::PimeInstruction, states::Transmutable};

    ///   - `u64`       The amount to transfer (without decimals).
    ///   - `u64`       The vault index.
    ///   - `u64`       The transfer index.
    ///   - `UnixTimestamp` Warmup period
    ///   - `UnixTimestamp` Validity period
#[repr(C)]
pub struct BookTransferInstructionData {
    pub discriminator: u8,
    amount: [u8; size_of::<i64>()],
    vault_index: [u8; size_of::<u64>()],
    transfer_index: [u8; size_of::<u64>()],
    warmup: [u8; size_of::<UnixTimestamp>()],
    validity: [u8; size_of::<UnixTimestamp>()],
}

impl BookTransferInstructionData {
    pub fn new(amount: u64, vault_index: u64, transfer_index: u64, warmup: UnixTimestamp, validity: UnixTimestamp) -> Self{
        Self { 
            discriminator: PimeInstruction::BookTransfer as u8, 
            amount: amount.to_le_bytes(),
            vault_index: vault_index.to_le_bytes(),
            transfer_index: transfer_index.to_le_bytes(),
            warmup: warmup.to_le_bytes(),
            validity: validity.to_le_bytes(),
        }
    }

    pub fn vault_index(&self) -> u64 {
        u64::from_le_bytes(self.vault_index)
    }

    pub fn transfer_index(&self) -> u64 {
        u64::from_le_bytes(self.transfer_index)
    }
}

/// # SAFETY : 
/// All fields are of u8 and therefore without padding.
unsafe impl Transmutable for BookTransferInstructionData {}
