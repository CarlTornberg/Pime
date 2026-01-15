use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq};
use pinocchio_token::state::TokenAccount;

use crate::{errors::PimeError, interface::instructions::close_vault_instruction::CloseVaultInstructionData, states::{Transmutable, VaultData, from_bytes}};

pub fn process_close_vault(accounts: &[AccountInfo], inst_data: &[u8]) -> ProgramResult {
    
    //      INSTRUCTION DESERIALAZATION

    let vault_index = if inst_data.len() < CloseVaultInstructionData::LEN - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    else {
        u64::from_le_bytes(unsafe { *(inst_data.as_ptr() as *const [u8; size_of::<u64>()]) }) 
    };
    
    let [authority_info, vault_info, vault_data_info, mint_info, token_program_info, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //      ACCOUNT SAFETY CHECKS

    if !authority_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !mint_info.is_owned_by(token_program_info.key()) {
        msg!("Mint is not owned by the supplied token program.");
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    let vault_data_pda = VaultData::get_vault_data_pda(authority_info.key(), vault_index, mint_info.key(), token_program_info.key());
    if !pubkey_eq(vault_data_info.key(), &vault_data_pda.0) {
        msg!("Incorrect Vault data PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data_info.lamports() == 0 {
        msg!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data_info.is_owned_by(&crate::ID) {
        msg!("Vault data is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault_data_info.is_writable() {
        msg!("Vault data is not writable.");
        return Err(ProgramError::Immutable);
    }

    let vault_pda = VaultData::get_vault_pda(authority_info.key(), vault_index, mint_info.key(), token_program_info.key());
    if !pubkey_eq(vault_info.key(), &vault_pda.0) {
        msg!("Incorrect Vault PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_info.lamports() == 0 {
        msg!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_info.is_owned_by(token_program_info.key()) {
        msg!("Vault is not owned by the supplied token program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault_info.is_writable() {
        msg!("Vault is not writable.");
        return Err(ProgramError::Immutable);
    }

    //      BUSINESS LOGIC

    // Make sure vault has no open transfers
    let vault_data = if vault_data_info.data_len() < VaultData::LEN {
        msg!("Vault data has insufficient data.");
        return Err(ProgramError::AccountDataTooSmall);
    }
    else {
        // SAFETY: Vault data is a valid representation and is of enough bytes.
        unsafe { &*(vault_data_info.data_ptr() as *const VaultData) }
    };
    if vault_data.open_transfers() != 0 {
        msg!("The vault has open transfers.");
        return Err(PimeError::VaultHasOpenTransfers.into());
    }

    // Check that vault is empty
    let vault = unsafe { TokenAccount::from_account_info_unchecked(vault_info)? };
    if vault.amount() != 0 {
        msg!("Vault is not empty.");
        return Err(PimeError::VaultIsNotEmpty.into());
    }

    // close vault account
    let vault_index_bytes = vault_index.to_le_bytes();
    let vault_bump = &[vault_pda.1];
    let vault_signer = VaultData::get_vault_signer_seeds(
        authority_info.key(),
        &vault_index_bytes, 
        mint_info.key(), 
        token_program_info.key(), 
        vault_bump);
    pinocchio_token::instructions::CloseAccount {
        account: vault_info,
        destination: authority_info,
        authority: vault_info
    }.invoke_signed(&[Signer::from(&vault_signer)])?;

    // close vault data account
    // SAFETY: Is not borrowed earlier. Transfer account is empty.
    unsafe {
        *authority_info.borrow_mut_lamports_unchecked() += vault_data_info.lamports();
        vault_data_info.close_unchecked();
    }

    ProgramResult::Ok(())
}
