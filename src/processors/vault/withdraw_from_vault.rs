use pinocchio::{ProgramResult, account_info::AccountInfo, msg, program_error::ProgramError, pubkey::pubkey_eq};

use crate::{errors::PimeError, interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData, processors::shared, shared::deserialize, states::VaultData};

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
    let [authority, vault_data, vault, to, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
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
