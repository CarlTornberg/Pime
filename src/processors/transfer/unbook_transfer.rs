use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, seeds};
use pinocchio_token::state::TokenAccount;

use crate::{errors::PimeError, interface::instructions::unbook_transfer_instruction::UnbookTransferInstructionData, states::{VaultData, transfer_data::TransferData}};

/// Closes a booked transfer account.
/// If the booking was never proceeded, the assets are transferred back to its owner.
pub fn unbook_transfer(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    
    // Deserialize intruction data
    let (vault_index, transfer_index) = if instruction_data.len() < size_of::<UnbookTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    else {
        (
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) })
        )
    };

    // Safety checks on accounts
    let [authority, vault_data, vault, transfer, deposit, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        msg!("Authority must be signer.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_data_pda = VaultData::get_vault_data_pda(
        /* authority */ authority.key(), 
        /* index */ vault_index, 
        /* mint */ mint.key(), 
        /* token_program */ token_program.key()
    );
    if !pubkey_eq(&vault_data_pda.0, vault_data.key()) {
        msg!("Vault data PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() == 0 {
        msg!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data.is_owned_by(&crate::ID) {
        msg!("Vault data is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    let vault_pda = VaultData::get_vault_pda(vault_data.key());
    if pubkey_eq(&vault_pda.0, vault.key()) {
        msg!("Vault PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault.lamports() == 0 {
        msg!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault.is_owned_by(token_program.key()) {
        msg!("Vault is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    let transfer_pda = TransferData::get_transfer_pda(vault_data.key(), &transfer_index);
    if !pubkey_eq(&transfer_pda.0, transfer.key()) {
        msg!("Transfer PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() == 0 {
        msg!("Transfer is not initilized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !transfer.is_owned_by(&crate::ID) {
        msg!("Transfer is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    let deposit_pda = TransferData::get_deposit_pda(transfer.key());
    if !pubkey_eq(&deposit_pda.0, deposit.key()) {
        msg!("Deposit PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() == 0 {
        msg!("Deposit is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !deposit.is_owned_by(token_program.key()){
        msg!("The deposit is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    if !pubkey_eq(token_program.key(), &pinocchio_token::ID) {
        msg!("Token program not supported.");
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint is not owned by the provided token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if vault.data_len() != TokenAccount::LEN {
        msg!("Vault does not contain enough data. Is it really a token account?");
        return Err(ProgramError::AccountDataTooSmall);
    }
    let vault_acc = unsafe {&*(vault.data_ptr() as *const TokenAccount)};
    let deposit_bump = &[deposit_pda.1];
    let deposit_seeds = seeds!(transfer.key(), deposit_bump);


    //      ** BUSINESS LOGIC **

    // Move assets from the deposit back to its vault
    pinocchio_token::instructions::Transfer {
        from: deposit,
        to: vault,
        authority: vault,
        amount: vault_acc.amount()
    }.invoke_signed(&[Signer::from(&deposit_seeds)])?;
    
    // Close the deposit account
    pinocchio_token::instructions::CloseAccount {
        account: deposit,
        destination: authority,
        authority: deposit,
    }.invoke_signed(&[Signer::from(&deposit_seeds)])?;
    
    // Close the transfer account
    // SAFETY: Is not borrowed earlier. Transfer account is empty.
    unsafe {
        *authority.borrow_mut_lamports_unchecked() += transfer.lamports();
        transfer.close_unchecked();
    }
    
    ProgramResult::Ok(())
}
