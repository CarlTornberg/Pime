use pinocchio::{program_error::ProgramError, pubkey::{Pubkey, find_program_address}, sysvars::{Sysvar, clock::{Clock, Epoch, UnixTimestamp}}};

use crate::states::Transmutable;

#[repr(C)]
pub struct TransferData {
    pub discriminator: u8,
    version: [u8; size_of::<u64>()],
    pub vault_data: Pubkey,
    pub destination: Pubkey,
    amount: [u8; size_of::<UnixTimestamp>()],
    created: [u8; size_of::<UnixTimestamp>()],
    created_epoch: [u8; size_of::<Epoch>()],
    warmup: [u8; size_of::<UnixTimestamp>()],
    validity: [u8; size_of::<UnixTimestamp>()],
}

impl TransferData {
    // pub const TRANSFER_SEED: &[u8] = b"transfer";
    // pub const DEPOSIT_SEED: &[u8] = b"deposit";

    pub fn new(vault_data: Pubkey, amount: u64, destination: Pubkey, warmup: UnixTimestamp, validity: UnixTimestamp) -> Result<Self, ProgramError> {
        let clock = Clock::get()?;
        Ok(Self { discriminator: 10u8, 
            version: 0u64.to_le_bytes(),
            vault_data, 
            amount: amount.to_le_bytes(),
            destination,
            created: clock.unix_timestamp.to_le_bytes(), 
            created_epoch: clock.epoch.to_le_bytes(),
            warmup: warmup.to_le_bytes(), 
            validity: validity.to_le_bytes()
        })
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn created(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.created)
    }

    pub fn created_epoch(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.created_epoch)
    }

    pub fn warmup(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.warmup)
    }

    pub fn validity(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.validity)
    }

    // Get the transfer's PDA.
    // Derived from the vault_data
    pub fn get_transfer_pda(vault_data: &Pubkey, transfer_index: &u64) -> (Pubkey, u8) {
        let program_id = &crate::ID;
        let seeds: &[&[u8]] = &[
            vault_data,
            &transfer_index.to_le_bytes(),
        ];
        find_program_address(seeds, program_id)
    }

    // Get the transfer's PDA.
    // Derived from the vault_data
    pub fn get_deposit_pda(transfer: &Pubkey) -> (Pubkey, u8) {
        let program_id = &crate::ID;
        let seeds: &[&[u8]] = &[
            transfer,
        ];
        find_program_address(seeds, program_id)
    }
}

/// # SAFETY
/// Struct does not contain padding.
unsafe impl Transmutable for TransferData { }
