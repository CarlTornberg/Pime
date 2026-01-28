use pinocchio::{AccountView, Address, ProgramResult, address::address_eq, cpi::Signer, error::ProgramError};
use solana_program_log::log;

use crate::{errors::PimeError, interface::instructions::deposit_to_vault_instruction::DepositToVaultInstructionData, processors, states::VaultData};

pub fn process_deposit_to_vault(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {

    // Extract instruction data
    let (vault_owner, vault_index, amount) = 
    // instruction size - discriminator
    if instruction_data.len() >= size_of::<DepositToVaultInstructionData>() - size_of::<u8>() { 
        (
            unsafe { &*(instruction_data.as_ptr() as *const Address) },
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<Address>()) as *const [u8; size_of::<u64>()]) } ),
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<Address>() + size_of::<u64>()) as *const [u8; size_of::<u64>()]) } ),
        )
    }
    else {
        return Err(ProgramError::InvalidInstructionData);
    };

    // Extract accounts
    let [from_authority, from, vault, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //      Validate account infos

    if !from_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !address_eq(token_program.address(), &pinocchio_token::ID) {
        return Err(PimeError::UnsupportedTokenProgram.into());
    } 

    if !mint.owned_by(token_program.address()) {
        log!("Mint is now owned by supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    let vault_pda = VaultData::find_vault_address(vault_owner, vault_index, mint.address(), token_program.address());
    if !address_eq(&vault_pda.0, vault.address()) {
        log!("Incorrect vault PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if !vault.is_writable() {
        log!("Vault needs to be writeable.");
        return Err(ProgramError::Immutable);
    }
    if vault.lamports() == 0 {
        if !address_eq(token_program.address(), &pinocchio_token::ID) {
            log!("Unsupported token program.");
            return Err(PimeError::UnsupportedTokenProgram.into());
        }
        let vault_index_bytes = vault_index.to_le_bytes();
        let vault_bump = &[vault_pda.1];
        let vault_signer_seeds = VaultData::vault_signer_seeds(
            vault_owner, 
            &vault_index_bytes, 
            mint.address(), 
            token_program.address(), 
            vault_bump);
        processors::shared::create_vault_account::create_vault_account(
            /* payer */ from_authority,
            /* vault */ vault,
            /* mint */ mint,
            /* token program */ token_program.address(),
            /* vault signer */ &Signer::from(&vault_signer_seeds)
        )?;
    } 
    else if !vault.owned_by(&pinocchio_token::ID) {
        log!("Vault is not owned by the supplied Token Program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      Business logic

    // Token transfer from, to vault
    pinocchio_token::instructions::Transfer {
        from,
        to: vault,
        authority: from_authority,
        amount
    }.invoke()?;

    ProgramResult::Ok(())
}
