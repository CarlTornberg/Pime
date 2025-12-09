use pinocchio::{ProgramResult, account_info::AccountInfo, program_error::ProgramError, pubkey::{Pubkey, pubkey_eq}};
use crate::states::{Vault, as_bytes};

/// Create new vault given a vault index, authority, mint (with corresponding token program), and
/// settings.
pub fn process_create_vault(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {

    // Validate instruction data
    let (
        index, 
        timeframe, 
        max_transactions, 
        max_lamports, 
        ) = if instrution_data.len() >= 
    size_of::<u64>() + // Vault index
    size_of::<u64>() + // Time frame
    size_of::<u64>() + // max_timeframe_transactions
    size_of::<u64>()   // max_timeframe_lamports
    {
        (
            u64::from_le_bytes(unsafe { *(instrution_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instrution_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instrution_data.as_ptr().add(size_of::<u64>() * 2) as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes(unsafe { *(instrution_data.as_ptr().add(size_of::<u64>() * 3) as *const [u8; size_of::<u64>()]) }),
        )
    }
    else {
        return Err(ProgramError::InvalidInstructionData);
    };
    
    // Validate accounts
    
    let [authority, vault_data, vault, mint, token_program, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_data_pda = Vault::get_vault_data_pda(authority.key(), index, mint.key(), token_program.key());
    if !pubkey_eq(&vault_data_pda.0, vault_data.key()) {
        return Err(ProgramError::Custom(0));
    }
    if vault_data.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let vault_pda = Vault::get_vault_pda(&vault_data_pda.0, mint.key(), token_program.key());
    if !pubkey_eq(&vault_pda.0, vault.key()) {
        return Err(ProgramError::Custom(0));
    }
    if vault.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    
    // Create accounts

    pinocchio_system::
        create_account_with_minimum_balance(
            /* account */ vault_data, 
            /* space */ size_of::<Vault>(), 
            /* owner */ &crate::ID, 
            /* payer */ authority, 
            /* rent sysvar */ None)?;
    
    pinocchio_system::
        create_account_with_minimum_balance(
            /* account */ vault, 
            /* space */ 0,
            /* owner */ &crate::ID, 
            /* payer */ authority, 
            /* rent sysvar */ None)?;

    if let Ok(mut vault_data_mut) = vault_data.try_borrow_mut_data() {
        let data = Vault::new(vault_data_pda.1, *authority.key());
        vault_data_mut.copy_from_slice(as_bytes(&data));
    }
    else {
        return Err(ProgramError::AccountBorrowFailed);
    };


    // (Optional) Deposit to vault

    ProgramResult::Ok(())
}
