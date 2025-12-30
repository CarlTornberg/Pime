use pinocchio::{pubkey::{Pubkey, find_program_address}, sysvars::clock::UnixTimestamp};
use crate::states::Transmutable;

#[repr(C)]
pub struct VaultData {
    pub(crate) discriminator: u8,
    version: [u8; size_of::<u64>()],
    pub(crate) authority: Pubkey,
    timeframe: [u8; size_of::<UnixTimestamp>()],
    max_amount: [u8; size_of::<u64>()],
    max_transactions: [u8; size_of::<u64>()],
    transfer_min_warmup: [u8; size_of::<UnixTimestamp>()],
    transfer_max_window: [u8; size_of::<UnixTimestamp>()],
    transaction_index: [u8; size_of::<u64>()],
}

unsafe impl Transmutable for VaultData { }

#[allow(dead_code)]
impl VaultData {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const VAULT_DATA_SEED: &[u8] = b"vault_data";
    pub const VAULT_STAKE_SEED: &[u8] = b"vault_stake";

    pub fn new(authority: Pubkey, timeframe: i64, max_amount: u64, max_transactions: UnixTimestamp, transfer_min_warmup: UnixTimestamp, transfer_max_window: UnixTimestamp) -> Self {
        Self { 
            discriminator: 0u8, 
            version: 1u64.to_le_bytes(), 
            authority, 
            timeframe: timeframe.to_le_bytes(), 
            max_amount: max_amount.to_le_bytes(),
            max_transactions: max_transactions.to_le_bytes(),
            transfer_min_warmup: transfer_min_warmup.to_le_bytes(),
            transfer_max_window: transfer_max_window.to_le_bytes(),
            transaction_index: 0u64.to_le_bytes()
        }
    }

    pub fn version(&self) -> u64 {
        u64::from_le_bytes(self.version)
    }

    pub(crate) fn set_version(&mut self, version: u64) {
        self.version = version.to_le_bytes();
    }

    pub fn timeframe(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.timeframe)
    }

    pub(crate) fn set_timeframe(&mut self, val: &UnixTimestamp) {
        self.timeframe = val.to_le_bytes();
    }

    pub fn max_transactions(&self) -> u64 {
        u64::from_le_bytes(self.max_transactions)
    }

    pub(crate) fn set_max_transactions(&mut self, val: &u64) {
        self.max_transactions = val.to_le_bytes();
    }

    pub fn max_amount(&self) -> u64 {
        u64::from_le_bytes(self.max_amount)
    }

    pub(crate) fn set_max_amount(&mut self, val: &u64) {
        self.max_amount = val.to_le_bytes();
    }

    pub fn transaction_index(&self) -> u64 {
        u64::from_le_bytes(self.transaction_index)
    }

    pub fn set_transaction_index(&mut self, val: &u64) {
        self.transaction_index = val.to_le_bytes();
    }

    pub fn transfer_min_warmup(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.transfer_min_warmup)
    }

    pub fn set_transfer_min_warmup(&mut self, val: &UnixTimestamp) {
        self.transfer_min_warmup = val.to_le_bytes();
    }

    pub fn transfer_max_window(&self) -> UnixTimestamp {
        UnixTimestamp::from_le_bytes(self.transfer_max_window)
    }

    pub fn set_transfer_max_window(&mut self, val: &UnixTimestamp) {
        self.transfer_max_window = val.to_le_bytes();
    }

    /// Calculates the vault data PDA with bump.
    /// If the vault is storing native token (SOL), do not provide mint and token program.
    /// If the vault is storing SPL tokens, provide the corresponding mint and token program.
    ///
    /// Index allows an author to have multiple vaults for a specific token
    /// This enabled additional fine grained control over an asset.
    pub fn get_vault_data_pda(authority: &Pubkey, index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        let program_id = &crate::ID;
        let seeds = &[
            VaultData::VAULT_DATA_SEED,
            authority,
            &index.to_le_bytes(),
            mint,
            token_program,
        ];
        find_program_address(seeds, program_id)
    }
    
    /// Get the Vault PDA, which is a ATA owner by the vault_data
    /// TODO Should this be a ATA or not?
    /// Pro: Predictable and follow the regular way of deriving it
    /// Con: Needs to be derived by the ATA ID, and calling the ADA does nothing
    /// else than checks that the ATA derivation is correct, then calls
    /// the token program to create the account, which is then owned by the token program.
    pub fn get_vault_pda(vault_data: &Pubkey) -> (Pubkey, u8) {
        // Is an ATA derived address.
        find_program_address(&[
            vault_data,
        ], 
            &crate::ID)
    }
    
}

impl VaultData {
    /// Get Vault data from the byte array.
    ///
    /// # SAFETY
    /// Bytes must be representation of a Vault
    /// Can be longer than Vault, however, all initial bytes must represent Vault
    pub unsafe fn from_account_data_bytes(bytes: &[u8]) -> &Self {
        let vault_bytes = &bytes[0..size_of::<VaultData>()];
        &*(vault_bytes.as_ptr() as *const VaultData)
    }

}

#[repr(C)]
pub struct VaultHistory {
    timestamp: [u8; size_of::<i64>()],
    amount: [u8; size_of::<u64>()],
}

/// # SAFETY
/// Struct does not contain padding.
unsafe impl Transmutable for VaultHistory { }

impl VaultHistory {
    pub fn new(timestamp: i64, amount: u64) -> Self {
        Self { timestamp: timestamp.to_le_bytes(), amount: amount.to_le_bytes() }
    }

    pub fn timestamp(&self) -> i64 {
        i64::from_le_bytes(self.timestamp)
    }

    pub fn set_timestamp(&mut self, val: i64) {
        self.timestamp = val.to_le_bytes();
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn set_amount(&mut self, val: u64) {
        self.amount = val.to_le_bytes();
    }
}
