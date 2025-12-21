use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, seeds};
use crate::{errors::PimeError, processors::shared, states::VaultData};

/// Create new vault given a vault index, authority, mint (with corresponding token program), and
/// settings.
pub fn process_create_vault(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    // Validate instruction data
    let (
        index, 
        timeframe, 
        max_transactions, 
        max_lamports, 
        ) = if instruction_data.len() >= 
    size_of::<u64>() + // vault index
    size_of::<i64>() + // time frame
    size_of::<u64>() + // max transactions (in the timeframe)
    size_of::<u64>()   // max lamports (in the timeframe)
    {
        (
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            i64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 2) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 3) as *const [u8; size_of::<u64>()]) }),
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

    let vault_data_pda = VaultData::get_vault_data_pda(authority.key(), index, mint.key(), token_program.key());
    if !pubkey_eq(&vault_data_pda.0, vault_data.key()) {
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !vault_data.is_writable() {
        return Err(ProgramError::Immutable);
    }

    let vault_data_pda_bump = &[vault_data_pda.1]; // prevent dropping
    let index_bytes = index.to_le_bytes(); // prevent dropping
    let vault_data_signer_seeds = seeds!(
        VaultData::VAULT_DATA_SEED,
        authority.key(),
        &index_bytes,
        mint.key(),
        token_program.key(),
        vault_data_pda_bump
    );
    shared::create_vault_data_account::process_create_vault_data_account(
        authority,
        vault_data,
        max_transactions,
        timeframe,
        max_lamports,
        Signer::from(&vault_data_signer_seeds),
    )?;
    
    //   Vault
    let vault_pda = VaultData::get_vault_pda(&vault_data_pda.0);
    if !pubkey_eq(&vault_pda.0, vault.key()) {
        return Err(PimeError::IncorrectPDA.into());
    }

    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }
    
    if vault.lamports() == 0 { // If account has not been initialized, init it
        shared::create_vault_account::create_vault_account(
            /* payer */ authority,
            /* vault */ vault,
            /* vault bump */ vault_pda.1,
            /* vault data pubkey */ vault_data.key(),
            /* mint */ mint,
            /* token program */ token_program.key(),
        )?;
    }
    else if !vault.is_owned_by(&pinocchio_token::ID) { // Force vault to be owned by token program
        msg!("Be aware, the vault is not owned by the token program.");
        // (TODO fix so that is supports other programs, but with safety (pre init attacks etc)
        return Err(PimeError::InvalidTokenProgram.into());
    }

    ProgramResult::Ok(())
}
