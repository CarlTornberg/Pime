use pinocchio::{ProgramResult, account_info::AccountInfo, msg, program_error::ProgramError, pubkey::{Pubkey, pubkey_eq}};

use crate::{errors::PimeError, interface::instructions::deposit_to_vault_instruction::DepositToVaultInstructionData, processors, states::VaultData};

pub fn process_deposit_to_vault(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    // Extract instruction data
    let (vault_owner, index, amount) = 
    // instruction size - discriminator
    if instruction_data.len() >= size_of::<DepositToVaultInstructionData>() - size_of::<u8>() { 
        (
            unsafe { &*(instruction_data.as_ptr() as *const Pubkey) },
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<Pubkey>()) as *const [u8; size_of::<u64>()]) } ),
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<Pubkey>() + size_of::<u64>()) as *const [u8; size_of::<u64>()]) } ),
        )
    }
    else {
        return Err(ProgramError::InvalidInstructionData);
    };

    // Extract accounts
    let [from_authority, from, vault, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Safety checks on accounts

    //      from account checks

    if !from_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    //      token program account checks

    if !pubkey_eq(token_program.key(), &pinocchio_token::ID) {
        return Err(PimeError::UnsupportedTokenProgram.into());
    } 

    //    Mint 
    
    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint is now owned by supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      vault data account checks

    let vault_data_pda = VaultData::get_vault_data_pda(vault_owner, index, mint.key(), token_program.key());
    let vault_pda = VaultData::get_vault_pda(&vault_data_pda.0);
    if !pubkey_eq(&vault_pda.0, vault.key()) {
        msg!("Invalid vault");
        return Err(PimeError::IncorrectPDA.into());
    }

    //      vault account checks
    
    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if vault.lamports() == 0 {
        processors::shared::create_vault_account::create_vault_account(
            /* payer */ from_authority,
            /* vault */ vault,
            /* vault bump */ vault_pda.1,
            /* vault data pubkey */ &vault_data_pda.0,
            /* mint */ mint,
            /* token program */ token_program.key(),
        )?;
    } 
    else if !vault.is_owned_by(&pinocchio_token::ID) {
        msg!("Vault is not owned by the Token Program.");
        return Err(ProgramError::InvalidAccountOwner);
    }


    // Token transfer from, to vault
    pinocchio_token::instructions::Transfer {
        from,
        to: vault,
        authority: from_authority,
        amount
    }.invoke()?;

    ProgramResult::Ok(())
}
