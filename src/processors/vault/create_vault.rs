use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, seeds};
use pinocchio_token::state::TokenAccount;
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
        msg!("Not enough instruction data. Did you include all fields?");
        return Err(ProgramError::InvalidInstructionData);
    };
    
    // Validate accounts
    
    let [authority, vault_data, vault, mint, token_program, _remaining @ ..] = accounts else {
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

    let vault_pda = Vault::get_vault_pda(&vault_data_pda.0);
    if !pubkey_eq(&vault_pda.0, vault.key()) {
        return Err(ProgramError::Custom(0));
    }
    if vault.lamports() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Create accounts
    let vault_data_pda_bump = &[vault_data_pda.1]; // prevent dropping
    let index_bytes = index.to_le_bytes(); // prevent dropping
    let vault_data_signer_seeds = seeds!(
        Vault::VAULT_DATA_SEED,
        authority.key(),
        &index_bytes,
        mint.key(),
        token_program.key(),
        vault_data_pda_bump
    );
    let vault_data_signer = Signer::from(&vault_data_signer_seeds);
    pinocchio_system::
        create_account_with_minimum_balance_signed(
            /* account */ vault_data, 
            /* space */ size_of::<Vault>(), 
            /* owner */ &crate::ID, 
            /* payer */ authority, 
            /* rent sysvar */ None,
            /* signer seeds */ &[vault_data_signer]
        )?;
    
    let vault_pda_bump = &[vault_pda.1]; // prevent dropping
    let vault_signer_seeds = seeds!(
        &vault_data_pda.0,
        vault_pda_bump
    );
    let vault_signer = Signer::from(&vault_signer_seeds);
    pinocchio_system::
        create_account_with_minimum_balance_signed(
            /* account */ vault, 
            /* space */ TokenAccount::LEN,
            /* owner */ token_program.key(), 
            /* payer */ authority, 
            /* rent sysvar */ None,
            /* signer seeds */ &[vault_signer],
        )?;

    let vault_signer = Signer::from(&vault_signer_seeds);
    pinocchio_token::instructions::InitializeAccount3 {
        account: vault,
        mint,
        owner: &vault_pda.0
    }.invoke_signed(&[vault_signer])?;

    // Set vault data account data
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
