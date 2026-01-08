use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, sysvars::clock::UnixTimestamp};
use crate::{errors::PimeError, processors::shared, states::VaultData};

/// Create new vault given a vault index, authority, mint (with corresponding token program), and
/// settings.
pub fn process_create_vault(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    // Validate instruction data
    let (
        vault_index, 
        timeframe, 
        max_transactions, 
        max_amount, 
        transfer_min_warmup, 
        tranfer_max_window, 
        ) = if instruction_data.len() >= 
    size_of::<u64>() + // vault index
    size_of::<i64>() + // time frame
    size_of::<u64>() + // max transactions (in the timeframe)
    size_of::<u64>() + // max amount (in the timeframe)
    size_of::<UnixTimestamp>() + // transfer min warmup 
    size_of::<UnixTimestamp>()   // transfer max_window 
    {
        (
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            i64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 2) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 3) as *const [u8; size_of::<u64>()]) }),
            UnixTimestamp::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 4) as *const [u8; size_of::<UnixTimestamp>()]) }),
            UnixTimestamp::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 5) as *const [u8; size_of::<UnixTimestamp>()]) }),
        )
    }
    else {
        msg!("Not enough instruction data. Did you include all fields?");
        return Err(ProgramError::InvalidInstructionData);
    };
    if timeframe < 0 {
        msg!("Timeframe must be > 0");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    let [authority, vault_data, vault, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //    Authority
    
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //    Mint 
    
    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint is now owned by supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //    Vault Data

    let vault_data_pda = VaultData::get_vault_data_pda(authority.key(), vault_index, mint.key(), token_program.key());
    if !pubkey_eq(&vault_data_pda.0, vault_data.key()) {
        msg!("Vault data PDA incorrect");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() != 0 {
        msg!("Vault data is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !vault_data.is_writable() {
        msg!("Vault data is not mutable.");
        return Err(ProgramError::Immutable);
    }

    let vault_data_pda_bump = &[vault_data_pda.1]; // prevent dropping
    let vault_index_bytes = vault_index.to_le_bytes();
    let vault_data_signer_seeds = VaultData::get_vault_data_signer_seeds(
        authority.key(), 
        &vault_index_bytes, 
        mint.key(), 
        token_program.key(), 
        vault_data_pda_bump);
    shared::create_vault_data_account::process_create_vault_data_account(
        authority,
        vault_data,
        max_transactions,
        timeframe,
        max_amount,
        transfer_min_warmup,
        tranfer_max_window,
        &Signer::from(&vault_data_signer_seeds),
    )?;
    
    //   Vault
    let vault_pda = VaultData::get_vault_pda(authority.key(), vault_index, mint.key(), token_program.key());
    if !pubkey_eq(&vault_pda.0, vault.key()) {
        return Err(PimeError::IncorrectPDA.into());
    }

    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }
    
    if vault.lamports() == 0 { // If account has not been initialized, init it
        let vault_bump = &[vault_pda.1];
        let vault_seeds = VaultData::get_vault_signer_seeds(
            authority.key(), 
            &vault_index_bytes, 
            mint.key(), 
            token_program.key(), 
            vault_bump);
        shared::create_vault_account::create_vault_account(
            authority,
            vault,
            mint,
            token_program.key(),
            &Signer::from(&vault_seeds),
        )?;
    }
    else if !vault.is_owned_by(&pinocchio_token::ID) { // Force vault to be owned by token program
        msg!("Be aware, the vault is not owned by the token program.");
        // (TODO fix so that is supports other programs, but with safety (pre init attacks etc)
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    ProgramResult::Ok(())
}
