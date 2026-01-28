use pinocchio::{AccountView, ProgramResult, address::address_eq, cpi::Signer, error::ProgramError, sysvars::clock::UnixTimestamp};
use solana_program_log::log;
use crate::{errors::PimeError, interface::instructions::create_vault_instruction::CreateVaultInstructionData, processors::shared, states::VaultData};

/// Create new vault given a vault index, authority, mint (with corresponding token program), and
/// settings.
pub fn process_create_vault(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {

    // Validate instruction data
    let (
        vault_index, 
        timeframe, 
        max_transactions, 
        max_amount, 
        allows_transfers,
        transfer_min_warmup, 
        tranfer_max_window, 
    ) = if instruction_data.len() < size_of::<CreateVaultInstructionData>() - size_of::<u8>() {
        log!("Not enough instruction data. Did you include all fields?");
        return Err(ProgramError::InvalidInstructionData);
    }
    else {
        (
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            i64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 2) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 3) as *const [u8; size_of::<u64>()]) }),
            unsafe { &*(instruction_data.as_ptr().add(size_of::<u64>() * 3 + size_of::<u8>())) },
            UnixTimestamp::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 4 + size_of::<u8>()) as *const [u8; size_of::<UnixTimestamp>()]) }),
            UnixTimestamp::from_le_bytes(unsafe { *(instruction_data.as_ptr().add(size_of::<u64>() * 5 + size_of::<u8>()) as *const [u8; size_of::<UnixTimestamp>()]) }),
        )
    };
    if timeframe < 0 {
        log!("Timeframe must be > 0");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    let [authority, vault_data, vault, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //      Validate account infos

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !mint.owned_by(token_program.address()) {
        log!("Mint is now owned by supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    let vault_data_pda = VaultData::find_vault_data_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(&vault_data_pda.0, vault_data.address()) {
        log!("Vault data PDA incorrect");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() != 0 {
        log!("Vault data is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !vault_data.is_writable() {
        log!("Vault data is not mutable.");
        return Err(ProgramError::Immutable);
    }

    let vault_data_pda_bump = &[vault_data_pda.1]; // prevent dropping
    let vault_index_bytes = vault_index.to_le_bytes();
    let vault_data_signer_seeds = VaultData::vault_data_signer_seeds(
        authority.address(), 
        &vault_index_bytes, 
        mint.address(), 
        token_program.address(), 
        vault_data_pda_bump);
    shared::create_vault_data_account::process_create_vault_data_account(
        authority,
        vault_data,
        max_transactions,
        timeframe,
        max_amount,
        *allows_transfers,
        transfer_min_warmup,
        tranfer_max_window,
        &Signer::from(&vault_data_signer_seeds),
    )?;
    
    let vault_pda = VaultData::find_vault_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(&vault_pda.0, vault.address()) {
        return Err(PimeError::IncorrectPDA.into());
    }
    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }
    if vault.lamports() == 0 { // If account has not been initialized, init it
        let vault_bump = &[vault_pda.1];
        let vault_seeds = VaultData::vault_signer_seeds(
            authority.address(), 
            &vault_index_bytes, 
            mint.address(), 
            token_program.address(), 
            vault_bump);
        shared::create_vault_account::create_vault_account(
            authority,
            vault,
            mint,
            token_program.address(),
            &Signer::from(&vault_seeds),
        )?;
    }
    else if !vault.owned_by(&pinocchio_token::ID) { // Force vault to be owned by token program
        log!("Be aware, the vault is not owned by the token program. This may be inteded.");
        // (TODO fix so that is supports other programs, but with safety (pre init attacks etc)
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    ProgramResult::Ok(())
}
