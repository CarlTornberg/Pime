use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::{Pubkey, pubkey_eq}, seeds, sysvars::{Sysvar, rent::Rent}};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{errors::PimeError, interface::instructions::book_transfer::BookTransferInstructionData, processors::shared::create_deposit_account::create_deposit_account, states::{VaultData, as_bytes, from_bytes, transfer_data::TransferData}};

/// Books a transfer and stores the assets in a temporary vault.
pub fn process_book_transfer(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    if instruction_data.len() < size_of::<BookTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // SAFETY: instruction data is long enough
    let (amount, destination, vault_index, transfer_index, warmup, validity) = unsafe { (
        u64::from_le_bytes( *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()])), 
        &*(instruction_data.as_ptr().add(size_of::<u64>()) as *const Pubkey), 
        u64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Pubkey>() + size_of::<u64>()) as *const [u8; size_of::<u64>()])),
        u64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Pubkey>() + 2 * size_of::<u64>()) as *const [u8; size_of::<u64>()])),
        i64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Pubkey>() + 2 * size_of::<u64>() + size_of::<i64>()) as *const [u8; size_of::<u64>()])),
        i64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Pubkey>() + 2 * size_of::<u64>() + 2 * size_of::<i64>()) as *const [u8; size_of::<u64>()])),
    ) 
    };

    if warmup < 0 {
        msg!("Warm-up must be positive.");
        return Err(ProgramError::InvalidInstructionData);
    }
    if validity < 0 {
        msg!("Validity must be positive.");
        return Err(ProgramError::InvalidInstructionData);
    }

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
    if !vault_data.is_owned_by(&crate::ID) {
        msg!("Vault data account is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if vault_data.data_len() < size_of::<VaultData>() {
        msg!("Vault data is of incorrect size.");
        return Err(ProgramError::AccountDataTooSmall);
    }
    // SAFETY: Vault data is read-only and is of enough bytes. 
    let vault_data_account = 
    unsafe { from_bytes::<VaultData>(&vault_data.borrow_data_unchecked()[..size_of::<VaultData>()]) }?;
    if vault_data_account.transfer_min_warmup() < warmup {
        msg!("The instructed warm-up violates the vaults min warm-up.");
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
    if !vault.is_owned_by(token_program.key()) {
        msg!("Vault account is not owned by the supplied token program.");
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

    let deposit_pda = TransferData::get_deposit_pda(transfer.key());
    if !pubkey_eq(deposit.key(), &deposit_pda.0) {
        msg!("Incorrect deposit PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() != 0 {
        msg!("The deposit is already in use");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    //      Create transfer account and assign data
    let transfer_bump = &[transfer_pda.1];
    let transfer_index_bytes = transfer_index.to_le_bytes();
    let transfer_seed = seeds!(&vault_data_pda.0, &transfer_index_bytes, transfer_bump);
    CreateAccount {
        from: authority,
        to: transfer,
        lamports: Rent::get()?.minimum_balance(size_of::<TransferData>()),
        space: size_of::<TransferData>() as u64,
        owner: &crate::ID,
    }.invoke_signed(&[Signer::from(&transfer_seed)])?;
    // SAFETY: Data is not previously borrowed and has the Transmutable trait.
    unsafe {
        core::slice::from_raw_parts_mut(
            transfer.data_ptr(), 
            size_of::<TransferData>()) }
        .copy_from_slice(as_bytes(
            &TransferData::new(
                /* vault data */ *vault_data.key(), 
                /* amount */ amount,
                /* destination */ *destination,
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
