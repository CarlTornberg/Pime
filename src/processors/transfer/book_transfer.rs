use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, seeds, sysvars::{Sysvar, rent::Rent}};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{errors::PimeError, processors::shared::create_deposit_account::create_deposit_account, states::{VaultData, as_bytes, from_bytes, transfer_data::TransferData}};

/// Books a transfer and stores the assets in a temporary vault.
pub fn process_book_transfer(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {
    

    let (amount, vault_index, transfer_index, warmup, validity) = (1,2,3,4,5);

    let [authority, vault_data, vault, transfer, deposit, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint not owned by supplied token program.");
        return Err(ProgramError::IllegalOwner);
    }

    let vault_data_pda = VaultData::get_vault_data_pda(authority.key(), vault_index, mint.key(), token_program.key());
    if !pubkey_eq(vault_data.key(), &vault_data_pda.0) {
        msg!("Incorrect vault data PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() == 0 {
        msg!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if vault_data.is_owned_by(&crate::ID) {
        msg!("Vault data account is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if vault_data.data_len() < size_of::<VaultData>() {
        msg!("Vault data is of incorrect size.");
        return Err(ProgramError::AccountDataTooSmall);
    }
    // SAFETY: Vault data is read-only and is of enough bytes. 
    let vault_data_account = unsafe { from_bytes::<VaultData>(vault.borrow_data_unchecked()) }?;
    if vault_data_account.transfer_min_warmup() > warmup {
        msg!("The instructed warm-up violated the vaults min warm-up.");
        return Err(PimeError::VaultWarmupViolation.into());
    }

    let vault_pda = VaultData::get_vault_pda(vault_data.key());
    if !pubkey_eq(vault.key(), &vault_pda.0) {
        msg!("Incorrect vault PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault.lamports() == 0 {
        msg!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if vault.is_owned_by(&crate::ID) {
        msg!("Vault account is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if vault.data_len() < TokenAccount::LEN {
        msg!("Vault is not of enough length. Is it really a token account?");
        return Err(ProgramError::AccountDataTooSmall);
    }
    // SAFETY vault is read-only by this call, and not used after the Token Program CPI.
    // let vault_account = unsafe { TokenAccount::from_bytes_unchecked(vault.borrow_data_unchecked()) };

    let transfer_pda = TransferData::get_transfer_pda(vault_data.key(), &transfer_index);
    if !pubkey_eq(transfer.key(), &transfer_pda.0) {
        msg!("Incorrect transfer PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() != 0 {
        msg!("A transfer is already booked.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !transfer.is_owned_by(&crate::ID) {
        msg!("Transfer account is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }

    let deposit_pda = TransferData::get_deposit_pda(transfer.key());
    if !pubkey_eq(deposit.key(), &deposit_pda.0) {
        msg!("Incorrect deposit PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() != 0 {
        msg!("The deposit is already in use");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !deposit.is_owned_by(token_program.key()) {
        msg!("Deposit is not owned by the provided token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      Create transfer account and assign data
    CreateAccount {
        from: authority,
        to: transfer,
        lamports: Rent::get()?.minimum_balance(size_of::<TransferData>()),
        space: size_of::<TransferData>() as u64,
        owner: &crate::ID,
    }.invoke();
    // SAFETY: Data is not previously borrowed and is represented by a valid format.
    unsafe {
        core::slice::from_raw_parts_mut(
            transfer.data_ptr(), 
            size_of::<TransferData>()) }
        .copy_from_slice(as_bytes(
            &TransferData::new(
                /* vault data */ *vault_data.key(), 
                /* amount */ amount, 
                /* warm-up */ warmup, 
                /* validity */ validity)?
        ));

    //      Create deposit token account
    create_deposit_account(
        /* payer */ authority, 
        /* deposit */ deposit, 
        /* transfer */ transfer, 
        /* deposit_bump */ &[deposit_pda.1], 
        /* mint */ mint, 
        /* token_program */ token_program.key())?;

    //      Transfer from vault to deposit
    let vault_pda_seed = &[vault_pda.1]; // Prevent dropping
    let vault_signer_seed = seeds!(vault_data.key(), vault_pda_seed);
    Transfer {
        from: vault,
        to: deposit,
        authority: vault,
        amount
    }.invoke_signed(&[Signer::from(&vault_signer_seed)])?;

    ProgramResult::Ok(())
}
