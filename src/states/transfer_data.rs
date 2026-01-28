use pinocchio::{
    Address, 
    cpi::Seed, 
    error::ProgramError, 
    instruction::seeds, 
    sysvars::{
        Sysvar, 
        clock::{
            Clock, 
            Epoch, 
            UnixTimestamp
        }
    }
};

use crate::states::Transmutable;

#[repr(C)]
pub struct TransferData {
    pub discriminator: u8,
    version: [u8; size_of::<u64>()],
    pub vault_data: Address,
    pub destination: Address,
    amount: [u8; size_of::<UnixTimestamp>()],
    created: [u8; size_of::<UnixTimestamp>()],
    created_epoch: [u8; size_of::<Epoch>()],
    warmup: [u8; size_of::<UnixTimestamp>()],
    validity: [u8; size_of::<UnixTimestamp>()],
}

impl TransferData {
    pub const TRANSFER_SEED: &[u8] = b"transfer";
    pub const DEPOSIT_SEED: &[u8] = b"deposit";

    pub fn new(vault_data: Address, amount: u64, destination: Address, warmup: UnixTimestamp, validity: UnixTimestamp) -> Result<Self, ProgramError> {
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
    pub fn find_transfer_address(authority: &Address, destination: &Address, vault_index: u64, transfer_index: u64, mint: &Address, token_program: &Address) -> (Address, u8) {
        let seeds: &[&[u8]] = &[
            Self::TRANSFER_SEED,
            &vault_index.to_le_bytes(),
            &transfer_index.to_le_bytes(),
            authority.as_array(),
            destination.as_array(),
            mint.as_array(),
            token_program.as_array(),
        ];
        Address::find_program_address(seeds, &crate::ID)
    }
    pub fn transfer_signer_seeds<'a>(
        authority: &'a Address, 
        destination: &'a Address, 
        vault_index: &'a [u8; size_of::<u64>()], 
        transfer_index: &'a [u8; size_of::<u64>()], 
        mint: &'a Address, 
        token_program: &'a Address, 
        bump: &'a [u8]) -> [Seed<'a>; 8] {
        seeds!(
            Self::TRANSFER_SEED,
            vault_index,
            transfer_index,
            authority.as_array(),
            destination.as_array(),
            mint.as_array(),
            token_program.as_array(),
            bump
        )
    }

    // Get the transfer's PDA.
    // Derived from the vault_data
    pub fn find_deposit_address(authority: &Address, destination: &Address, vault_index: u64, transfer_index: u64, mint: &Address, token_program: &Address) -> (Address, u8) {
        let seeds: &[&[u8]] = &[
            Self::DEPOSIT_SEED,
            &vault_index.to_le_bytes(),
            &transfer_index.to_le_bytes(),
            authority.as_array(),
            destination.as_array(),
            mint.as_array(),
            token_program.as_array(),
        ];
        Address::find_program_address(seeds, &crate::ID)
    }
    pub fn deposit_signer_seeds<'a>(
        authority: &'a Address, 
        destination: &'a Address, 
        vault_index: &'a [u8; size_of::<u64>()], 
        transfer_index: &'a [u8; size_of::<u64>()], 
        mint: &'a Address, 
        token_program: &'a Address, 
        bump: &'a [u8]) -> [Seed<'a>; 8] {
        seeds!(
            Self::DEPOSIT_SEED,
            vault_index,
            transfer_index,
            authority.as_array(),
            destination.as_array(),
            mint.as_array(),
            token_program.as_array(),
            bump
        )
    }
}

/// # SAFETY
/// Struct does not contain padding.
unsafe impl Transmutable for TransferData {
    const LEN: usize = size_of::<Self>();
}
