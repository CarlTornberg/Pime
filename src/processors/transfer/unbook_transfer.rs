use pinocchio::{AccountView, Address, ProgramResult, address::address_eq, cpi::Signer, error::ProgramError};
use solana_program_log::log;
use pinocchio_token::state::TokenAccount;

use crate::{errors::PimeError, interface::instructions::unbook_transfer_instruction::UnbookTransferInstructionData, states::{VaultData, transfer_data::TransferData}};

/// Closes a booked transfer account.
/// If the booking was never proceeded, the assets are transferred back to its owner.
pub fn unbook_transfer(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    
    // Deserialize instruction data
    let (vault_index, transfer_index, destination) = if instruction_data.len() < size_of::<UnbookTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    else {
        (
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()]) }),
            u64::from_le_bytes( unsafe { *(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()]) }),
            unsafe {&*(instruction_data.as_ptr().add(2 * size_of::<u64>()) as *const Address)}
        )
    };

    // Safety checks on accounts
    let [authority, vault, vault_data, transfer, deposit, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        log!("Authority must be signer.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_pda = VaultData::find_vault_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(&vault_pda.0, vault.address()) {
        log!("Vault PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault.lamports() == 0 {
        log!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault.owned_by(token_program.address()) {
        log!("Vault is not owned by the supplied token program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault.is_writable() {
        log!("Vault is not writeable.");
        return Err(ProgramError::Immutable);
    }

    let vault_data_pda = VaultData::find_vault_data_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(&vault_data_pda.0, vault_data.address()) {
        log!("Vault data PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() == 0 {
        log!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data.owned_by(&crate::ID) {
        log!("Vault data is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault_data.is_writable() {
        log!("Vault data is not writeable.");
        return Err(ProgramError::Immutable);
    }

    let transfer_pda = TransferData::find_transfer_address(
        authority.address(), 
        destination, 
        vault_index, 
        transfer_index, 
        mint.address(), 
        token_program.address()
    );
    if !address_eq(&transfer_pda.0, transfer.address()) {
        log!("Transfer PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() == 0 {
        log!("Transfer is not initilized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !transfer.owned_by(&crate::ID) {
        log!("Transfer is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    let deposit_pda = TransferData::find_deposit_address(authority.address(), destination, vault_index, transfer_index, mint.address(), token_program.address());
    if !address_eq(&deposit_pda.0, deposit.address()) {
        log!("Deposit PDA incorrect.");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() == 0 {
        log!("Deposit is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !deposit.owned_by(token_program.address()){
        log!("The deposit is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    if !address_eq(token_program.address(), &pinocchio_token::ID) {
        log!("Token program not supported.");
        return Err(PimeError::UnsupportedTokenProgram.into());
    }

    if !mint.owned_by(token_program.address()) {
        log!("Mint is not owned by the provided token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if vault.data_len() != TokenAccount::LEN {
        log!("Vault does not contain enough data. Is it really a token account?");
        return Err(ProgramError::AccountDataTooSmall);
    }
    let vault_acc = unsafe {&*(vault.data_ptr() as *const TokenAccount)};

    let vault_index_bytes = vault_index.to_le_bytes();
    let transfer_index_bytes = transfer_index.to_le_bytes();
    let deposit_bump = &[deposit_pda.1];
    let deposit_seeds = TransferData::deposit_signer_seeds(
        authority.address(), 
        destination,
        &vault_index_bytes,
        & transfer_index_bytes, 
        mint.address(), 
        token_program.address(), 
        deposit_bump
    );


    //      ** BUSINESS LOGIC **

    // Move assets from the deposit back to its vault
    pinocchio_token::instructions::Transfer {
        from: deposit,
        to: vault,
        authority: deposit,
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
        authority.set_lamports(authority.lamports() + transfer.lamports());
        transfer.close_unchecked();
    }

    // Decrement open transfers from the vault data.
    // SAFETY: Only mutable here. Vault data bytes are a valid representation of VaultData
    let vault_data_mut = unsafe { &mut *(vault_data.data_ptr() as *mut VaultData) };
    vault_data_mut.set_open_transfers(vault_data_mut.open_transfers() - 1);
    
    ProgramResult::Ok(())
}
