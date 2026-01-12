use pinocchio::{ProgramResult, account_info::AccountInfo, msg, program_error::ProgramError, pubkey::pubkey_eq, sysvars::{Sysvar, clock::Clock}};

use crate::{errors::PimeError, interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData, processors::shared, states::{Transmutable, VaultData, VaultHistory, as_bytes, from_bytes}};

pub fn process_withdraw_from_vault(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {

    // Extract instruction data
    let (vault_index, amount) = if instrution_data.len() >= size_of::<WithdrawFromVaultInstructionData>() - size_of::<u8>() {
        (
            u64::from_le_bytes( unsafe { *(instrution_data.as_ptr() as *const [u8; size_of::<u64>()]) } ),
            u64::from_le_bytes( unsafe { *(instrution_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) } )
        )
    }
    else {
        return Err(ProgramError::InvalidInstructionData);
    };
    
    // Extract accounts
    let [authority_info, vault_data_info, vault_info, to_info, mint_info, token_program_info, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !mint_info.is_owned_by(token_program_info.key()) {
        msg!("Mint is not owned by the supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    let vault_data_pda = VaultData::get_vault_data_pda(authority_info.key(), vault_index, mint_info.key(), token_program_info.key());
    if !pubkey_eq(vault_data_info.key(), &vault_data_pda.0) {
        msg!("Vault data PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if !vault_data_info.is_writable() {
        msg!("Vault data is not writeable.");
        return Err(ProgramError::Immutable);
    }
    if vault_data_info.lamports() == 0 {
        msg!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data_info.is_owned_by(&crate::ID) {
        msg!("Vault data is not owned by this program.");
         return Err(ProgramError::IllegalOwner);
    }
    if vault_data_info.data_len() < size_of::<VaultData>() {
        msg!("Incorrect vault data len.");
        return Err(ProgramError::InvalidAccountData);
    }

    let vault_pda = VaultData::get_vault_pda(authority_info.key(), vault_index, mint_info.key(), token_program_info.key());
    if !pubkey_eq(vault_info.key(), &vault_pda.0) {
        msg!("Vault PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if !vault_info.is_writable() {
        msg!("Vault is not writeable.");
        return Err(ProgramError::Immutable);
    }
    if vault_info.lamports() == 0 {
        msg!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_info.is_owned_by(token_program_info.key()) {
        msg!("Vault is not owned by the supplied token program.");
         return Err(PimeError::UnsupportedTokenProgram.into());
    }

    // Check if the vault can withdraw
    // SAFETY: Vault data is not borrowed before.
    let vault_data_mut = unsafe {
        &mut *(vault_data_info.data_ptr() as *mut VaultData)
    };

    // Loop all data beyond VaultData to check previous withdraws.
    // SAFETY: Vault data's continued data is its history and is 
    let new_history = unsafe { VaultData::can_withdraw(
        // max_transactions * VaultHistory::LEN long, where VaultHistory is Transmutable
        /* data ptr */ vault_data_info.data_ptr().add(VaultData::LEN), 
        /* now */ Clock::get()?.unix_timestamp, 
        /* last_index */ vault_data_mut.transaction_index(),
        /* amount */ amount,
        /* max transactions */ vault_data_mut.max_transactions(),
        /* max amount */ vault_data_mut.max_amount(),
        /* time frame */ vault_data_mut.timeframe())? };
    
    shared::transfer::transfer(
        /* authority */ authority_info, 
        /* vault_data */ vault_data_info, 
        /* vault */ vault_info, 
        /* to */ to_info, 
        /* mint */ mint_info, 
        /* token_program */ token_program_info,
        /* amount */ amount,
        /* vault index */ vault_index,
    )?;

    let next_index = 
        if vault_data_mut.transaction_index() == vault_data_mut.max_transactions() - 1 { 0 } 
        else { vault_data_mut.transaction_index() + 1 };

    // Write new history to vault_data account
    // SAFETY: Data is only borrowed here, both read and write.
    // Data written is of type Transmutable and both slice and data is of same length.
    unsafe {
        core::slice::from_raw_parts_mut(
            vault_data_info.data_ptr().add(VaultHistory::LEN * (next_index as usize)), 
            VaultHistory::LEN)
            .copy_from_slice(as_bytes(&new_history));
    }

    // Write new index to vault_data
    vault_data_mut.set_transaction_index(&next_index);
    unsafe {
        core::slice::from_raw_parts_mut(
            vault_data_info.data_ptr().add(VaultHistory::LEN * (next_index as usize)), 
            VaultHistory::LEN)
            .copy_from_slice(as_bytes(&new_history));
    }
    
    ProgramResult::Ok(())
}
