use pinocchio::{pubkey::{Pubkey, find_program_address}};

use crate::states::Transmutable;

#[repr(C)]
pub struct Vault {
    pub(crate) discriminator: u8,
    version: [u8; size_of::<u64>()],
    pub(crate) bump: u8,
    pub(crate) authority: Pubkey,
    last_used: [u8; size_of::<u64>()],
    cooldown: [u8; size_of::<u64>()],
    warmup: [u8; size_of::<u64>()],
}

unsafe impl Transmutable for Vault { }

impl Vault {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const VAULT_DATA_SEED: &[u8] = b"vault_data";
    pub const VAULT_STAKE_SEED: &[u8] = b"vault_stake";

    pub fn new(bump: u8, authority: Pubkey) -> Self {
        Self { 
            discriminator: 0u8, 
            version: 0u64.to_le_bytes(), 
            bump, 
            authority, 
            last_used: 0u64.to_le_bytes(), 
            cooldown: 0u64.to_le_bytes(), 
            warmup: 0u64.to_le_bytes()
        }
    }

    pub fn version(&self) -> u64 {
        u64::from_le_bytes(self.version)
    }

    pub(crate) fn set_version(&mut self, version: u64) {
        self.version.copy_from_slice(&version.to_le_bytes());
    }

    pub fn last_used(&self) -> u64 {
        u64::from_le_bytes(self.last_used)
    }

    pub(crate) fn set_last_used(&mut self, last_used: &u64) {
        self.last_used.copy_from_slice(&last_used.to_le_bytes());
    }

    pub fn cooldown(&self) -> u64 {
        u64::from_le_bytes(self.cooldown)
    }

    pub(crate) fn set_cooldown(&mut self, cooldown: &u64) {
        self.last_used.copy_from_slice(&cooldown.to_le_bytes());
    }

    pub fn warmup(&self) -> u64 {
        u64::from_le_bytes(self.warmup)
    }

    pub(crate) fn set_warmup(&mut self, warmup: &u64) {
        self.warmup.copy_from_slice(&warmup.to_le_bytes());
    }


    /// Calculates the vault data PDA with bump.
    /// If the vault is storing native token (SOL), do not provide mint and token program.
    /// If the vault is storing SPL tokens, provide the corresponding mint and token program.
    ///
    /// Index allows an author to have multiple vaults for a specific token
    /// This enabled additional fine grained control over an asset.
    pub fn get_vault_data_pda(author: &Pubkey, index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        find_program_address(&[
            Vault::VAULT_DATA_SEED,
            author,
            &index.to_le_bytes(),
            mint,
            token_program,
        ], 
            &crate::ID)
    }

    /// Get the Vault PDA, which is a ATA owner by the vault_data
    /// TODO Should this be a ATA or not?
    /// Pro: Predictable and follow the regular way of deriving it
    /// Con: Needs to be derived by the ATA ID, and calling the ADA does nothing
    /// else than checks that the ATA derivation is correct, then calls
    /// the token program to create the account, which is then owned by the token program.
    pub fn get_vault_pda(vault_data: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        // Is an ATA drived address.
        find_program_address(
            &[
                vault_data.as_slice(),
            ], 
            &crate::ID)
    }
    
    /// Calculates the vault stake PDA with bump.
    /// Derived from the vault data PDA.
    pub fn get_vault_stake_pda(vault_data: &Pubkey) -> (Pubkey, u8) {
        // System program account holding native SOL
        find_program_address(&[
            Vault::VAULT_STAKE_SEED,
            vault_data,
        ], 
            &crate::ID)
    }

    /// Packs a vault to its byte format.
    pub fn pack(&self, buf: &mut [u8; size_of::<Vault>()]) {
        buf.copy_from_slice(
            // # SAFETY: Vault is Transmutable
            unsafe {
                core::slice::from_raw_parts(
                    self as *const Self as *const u8,
                    size_of::<Vault>()
                )
            }
        );
    }
}
