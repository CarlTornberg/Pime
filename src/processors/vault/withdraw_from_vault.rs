use pinocchio::{ProgramResult, account_info::AccountInfo, msg, program_error::ProgramError, pubkey::pubkey_eq, sysvars::{Sysvar, clock::Clock}};

use crate::{errors::PimeError, interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData, processors::shared, states::{VaultData, VaultHistory, from_bytes}};

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
    if vault_data_info.is_writable() {
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
    if vault_info.is_writable() {
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
    let vault_data = from_bytes::<VaultData>(unsafe {&vault_data_info.borrow_data_unchecked()[.. size_of::<VaultData>()] })?;

    // Loop all data beyond VaultData to check previous withdraws.

    // SAFETY: Data beyond size_of::<VaultData> is not borrowed elsewhere, and the remaining data
// is of type Transmutable.
    let vault_data_ptr = unsafe { vault_data_info.data_ptr() };
    let mut history;
    let mut tot_amount: u64 = 0;
    let mut index = vault_data.transaction_index();
    let now = Clock::get()?.unix_timestamp;

    for i in [..vault_data.max_transactions()] {
        history = unsafe { &*(vault_data_ptr.add(vault_data.transaction_index() as usize * size_of::<VaultHistory>()) as *const VaultHistory) };

        if now < history.timestamp() + vault_data.timeframe() {
            tot_amount.strict_add(history.amount()); 
            
            index += 1;
            if index == vault_data.max_transactions() {
                index = 0;
            }
            continue;
        }
        else {
            // Empty slot found
            Ok(());
        }
    }
    
    // Check that the to account is the account that it can withdraw to.

    // Transfer/withdraw assets.
    
    // Add the withdraw to the vault_data bytes.
    
    shared::transfer::transfer(
        /* authority */ authority, 
        /* vault_data */ vault_data, 
        /* vault */ vault, 
        /* to */ to, 
        /* mint */ mint, 
        /* token_program */ token_program,
        /* amount */ amount,
        /* vault index */ vault_index,
    )?;
    
    ProgramResult::Ok(())
}
