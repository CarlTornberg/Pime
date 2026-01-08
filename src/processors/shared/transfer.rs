use pinocchio::{account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, sysvars::{Sysvar, clock::Clock}};

use crate::{errors::PimeError, states::{VaultData, VaultHistory, from_bytes}};

pub fn transfer(
    authority: &AccountInfo, 
    vault_data: &AccountInfo, 
    vault: &AccountInfo, 
    to: &AccountInfo, 
    mint: &AccountInfo, 
    token_program: &AccountInfo, 
    amount: u64, 
    vault_index: u64) -> Result<(), ProgramError> {
    // Perform safety check
    
    //      Authority 

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Is this even needed? Is it possible to perform attack signing as a non-initialized account?
    if authority.lamports() == 0 {
        return Err(ProgramError::UninitializedAccount);
    }

    //    Token Program

    if !pubkey_eq(token_program.key(), &pinocchio_token::ID) {
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    //    Mint 
    
    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint is not owned by supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      To 

    if to.lamports() == 0 {
        msg!("Receiving account is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }

    //      Vault Data
    
    if vault_data.lamports() == 0 {
        msg!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }

    if !vault_data.is_owned_by(&crate::ID) {
        return Err(ProgramError::IllegalOwner);
    }

    if !vault_data.is_writable() {
        return Err(ProgramError::Immutable);
    }

    let vault_data_pda = VaultData::get_vault_data_pda(authority.key(), vault_index, mint.key(), token_program.key());
    if !pubkey_eq(vault_data.key(), &vault_data_pda.0) {
        return Err(PimeError::IncorrectPDA.into());
    }

    //     Vault

    let vault_pda = VaultData::get_vault_pda(authority.key(), vault_index, mint.key(), token_program.key());
    if !pubkey_eq(vault.key(), &vault_pda.0) {
        return Err(PimeError::IncorrectPDA.into());
    }

    if vault.lamports() == 0 {
        return Err(ProgramError::UninitializedAccount);
    }

    if !vault.is_owned_by(token_program.key()) {
        msg!("Vault is not owned by the supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if vault_data.data_len() < size_of::<VaultData>() {
        return Err(ProgramError::InvalidAccountData);
    }

    //      Vault Data bytes deserialization

    // SAFETY: Data is not previously borrowed and is represented by a valid format.
    let vault_data_bytes = unsafe {
        core::slice::from_raw_parts (
            vault_data.data_ptr(), 
            size_of::<VaultData>())
    };
    let vault_data_desed = from_bytes::<VaultData>(vault_data_bytes)?;
    // if pubkey_eq(&vault_data_desed.authority, authority.key()) { // Probably redundant as the authority needs to be signer, and vault is derived from the authority.
    //     return Err(PimeError::AuthorityError.into());
    // }

    let max_transactions = vault_data_desed.max_transactions() as usize;
    // SAFETY: Remaining bytes of vault data is represented as VaultHistory
    let vault_history_bytes = unsafe { 
        core::slice::from_raw_parts_mut(
            vault_data.data_ptr().add(size_of::<VaultData>()), 
            max_transactions * size_of::<VaultHistory>()) 
    };

    let mut i = vault_data_desed.transaction_index() as usize;
    let mut vault_history: &mut VaultHistory;
    let now = Clock::get()?.unix_timestamp;
    let timeframe = vault_data_desed.timeframe();
    let mut tot_timeframe_amount = 0u64;
    let max_amount = vault_data_desed.max_amount();

    for _ in 0..max_transactions {
        i += 1;
        if i == max_transactions { i = 0; }

        // SAFETY: Bytes are represented as VaultHistory and are not mut borrowed.
        vault_history = unsafe { 
            &mut *(vault_history_bytes.as_mut_ptr().add(i * size_of::<VaultHistory>()) as *mut VaultHistory) 
        };

        if now + timeframe < vault_history.timestamp() {
            tot_timeframe_amount += vault_history.amount();
            if tot_timeframe_amount + amount > max_amount {
                return Err(PimeError::WithdrawLimitReachedAmount.into());
            }
        }
        else {
            // Found empty slot in the data.
            if tot_timeframe_amount + amount > max_amount {
                return Err(PimeError::WithdrawLimitReachedAmount.into());
            }

            let vault_bump = &[vault_pda.1];
            let vault_index_bytes = vault_index.to_le_bytes();
            let vault_signer_seeds = VaultData::get_vault_signer_seeds(
                authority.key(), 
                &vault_index_bytes,
                mint.key(), 
                token_program.key(), 
                vault_bump
            );
            return pinocchio_token::instructions::Transfer {
                from: vault,
                to, 
                authority: vault,
                amount
            }.invoke_signed(&[Signer::from(&vault_signer_seeds)]);
        }
    }

    // No empty slot found
    Err(PimeError::WithdrawLimitReachedTransactions.into())
}
