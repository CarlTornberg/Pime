use pinocchio::{instruction::Seed, program_error::ProgramError, pubkey::{Pubkey, find_program_address}, seeds, sysvars::{Sysvar, clock::{Clock, Epoch, UnixTimestamp}}};

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
    pub const TRANSFER_SEED: &[u8] = b"transfer";
    pub const DEPOSIT_SEED: &[u8] = b"deposit";

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
    pub fn get_transfer_pda(authority: &Pubkey, destination: &Pubkey, vault_index: u64, transfer_index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[
            Self::TRANSFER_SEED,
            &vault_index.to_le_bytes(),
            &transfer_index.to_le_bytes(),
            authority,
            destination,
            mint,
            token_program,
        ];
        find_program_address(seeds, &crate::ID)
    }
    pub fn get_transfer_signer_seeds<'a>(
        authority: &'a Pubkey, 
        destination: &'a Pubkey, 
        vault_index: &'a [u8; size_of::<u64>()], 
        transfer_index: &'a [u8; size_of::<u64>()], 
        mint: &'a Pubkey, 
        token_program: &'a Pubkey, 
        bump: &'a [u8]) -> [Seed<'a>; 8] {
        seeds!(
            Self::TRANSFER_SEED,
            vault_index,
            transfer_index,
            authority,
            destination,
            mint,
            token_program,
            bump
        )
    }

    // Get the transfer's PDA.
    // Derived from the vault_data
    pub fn get_deposit_pda(authority: &Pubkey, destination: &Pubkey, vault_index: u64, transfer_index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[
            Self::DEPOSIT_SEED,
            &vault_index.to_le_bytes(),
            &transfer_index.to_le_bytes(),
            authority,
            destination,
            mint,
            token_program,
        ];
        find_program_address(seeds, &crate::ID)
    }
    pub fn get_deposit_signer_seeds<'a>(
        authority: &'a Pubkey, 
        destination: &'a Pubkey, 
        vault_index: &'a [u8; size_of::<u64>()], 
        transfer_index: &'a [u8; size_of::<u64>()], 
        mint: &'a Pubkey, 
        token_program: &'a Pubkey, 
        bump: &'a [u8]) -> [Seed<'a>; 8] {
        seeds!(
            Self::DEPOSIT_SEED,
            vault_index,
            transfer_index,
            authority,
            destination,
            mint,
            token_program,
            bump
        )
    }
}

/// # SAFETY
/// Struct does not contain padding.
unsafe impl Transmutable for TransferData {
    const LEN: usize = size_of::<Self>();
}
