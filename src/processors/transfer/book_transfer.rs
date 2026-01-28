use pinocchio::{AccountView, Address, ProgramResult, address::address_eq, cpi::Signer, error::ProgramError, sysvars::{Sysvar, rent::Rent}};
use solana_program_log::log;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{errors::PimeError, interface::instructions::book_transfer::BookTransferInstructionData, processors::shared::create_deposit_account::create_deposit_account, states::{VaultData, as_bytes, from_bytes, transfer_data::TransferData}};

/// Books a transfer and stores the assets in a temporary vault.
pub fn process_book_transfer(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {

    if instruction_data.len() < size_of::<BookTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    // SAFETY: instruction data is long enough
    let (amount, destination, vault_index, transfer_index, warmup, validity) = unsafe { (
        u64::from_le_bytes( *(instruction_data.as_ptr() as *const [u8; size_of::<u64>()])), 
        &*(instruction_data.as_ptr().add(size_of::<u64>()) as *const Address), 
        u64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Address>() + size_of::<u64>()) as *const [u8; size_of::<u64>()])),
        u64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Address>() + 2 * size_of::<u64>()) as *const [u8; size_of::<u64>()])),
        i64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Address>() + 2 * size_of::<u64>() + size_of::<i64>()) as *const [u8; size_of::<u64>()])),
        i64::from_le_bytes( *(instruction_data.as_ptr().add(size_of::<Address>() + 2 * size_of::<u64>() + 2 * size_of::<i64>()) as *const [u8; size_of::<u64>()])),
    ) 
    };

    if warmup < 0 {
        log!("Warm-up must be positive.");
        return Err(ProgramError::InvalidInstructionData);
    }
    if validity < 0 {
        log!("Validity must be positive.");
        return Err(ProgramError::InvalidInstructionData);
    }

    let [authority, vault_data, vault, transfer, deposit, mint, token_program, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !mint.owned_by(token_program.address()) {
        log!("Mint not owned by supplied token program.");
        return Err(ProgramError::IllegalOwner);
    }

    let vault_data_pda = VaultData::find_vault_data_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(vault_data.address(), &vault_data_pda.0) {
        log!("Incorrect vault data PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() == 0 {
        log!("Vault data is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data.owned_by(&crate::ID) {
        log!("Vault data account is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault_data.is_writable() {
        log!("Vault data is not writable.");
        return Err(ProgramError::Immutable);
    }
    if vault_data.data_len() < size_of::<VaultData>() {
        log!("Vault data is of incorrect size.");
        return Err(ProgramError::AccountDataTooSmall);
    }
    // SAFETY: Vault data is not borrowed before this. 
    let vault_data_account = 
    unsafe { from_bytes::<VaultData>(&vault_data.borrow_unchecked()[..size_of::<VaultData>()]) }?;
    if vault_data_account.transfer_min_warmup() < warmup {
        log!("The instructed warm-up violates the vaults min warm-up.");
        return Err(PimeError::VaultWarmupViolation.into());
    }

    let vault_pda = VaultData::find_vault_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(vault.address(), &vault_pda.0) {
        log!("Incorrect vault PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault.lamports() == 0 {
        log!("Vault is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault.owned_by(token_program.address()) {
        log!("Vault account is not owned by the supplied token program.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault.is_writable() {
        log!("Vault is not writeable.");
        return Err(ProgramError::Immutable);
    }
    if vault.data_len() < TokenAccount::LEN {
        log!("Vault is not of enough length. Is it really a token account?");
        return Err(ProgramError::AccountDataTooSmall);
    }
    // SAFETY vault is read-only by this call, and not used after the Token Program CPI.
    // let vault_account = unsafe { TokenAccount::from_bytes_unchecked(vault.borrow_data_unchecked()) };

    let transfer_pda = TransferData::find_transfer_address(authority.address(), destination, vault_index, transfer_index, mint.address(), token_program.address());
    if !address_eq(transfer.address(), &transfer_pda.0) {
        log!("Incorrect transfer PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() != 0 {
        log!("A transfer is already booked.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !transfer.is_writable() {
        log!("Transfer is not writable.");
        return Err(ProgramError::Immutable);
    }

    let deposit_pda = TransferData::find_deposit_address(authority.address(), destination, vault_index, transfer_index, mint.address(), token_program.address());
    if !address_eq(deposit.address(), &deposit_pda.0) {
        log!("Incorrect deposit PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() != 0 {
        log!("The deposit is already in use");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !deposit.is_writable() {
        log!("Deposit is not writable.");
        return Err(ProgramError::Immutable);
    }

    //      Create transfer account and assign data
    let transfer_index_bytes = transfer_index.to_le_bytes();
    let vault_index_bytes = vault_index.to_le_bytes();
    let transfer_bump = &[transfer_pda.1];
    let transfer_seed = TransferData::transfer_signer_seeds(
        authority.address(), 
        destination, 
        &vault_index_bytes, 
        &transfer_index_bytes, 
        mint.address(), 
        token_program.address(), 
        transfer_bump
    );
    CreateAccount {
        from: authority,
        to: transfer,
        lamports: Rent::get()?.try_minimum_balance(size_of::<TransferData>())?,
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
                /* vault data */ vault_data.address().clone(), 
                /* amount */ amount,
                /* destination */ destination.clone(),
                /* warm-up */ warmup, 
                /* validity */ validity)?
        ));

    //      Create deposit token account
    let deposit_bump = &[deposit_pda.1];
    let deposit_signer_seeds = TransferData::deposit_signer_seeds(
        authority.address(), 
        destination, 
        &vault_index_bytes, 
        &transfer_index_bytes,
        mint.address(), 
        token_program.address(), 
        deposit_bump
    );
    create_deposit_account(
        /* payer */ authority,
        /* deposit */ deposit,
        /* mint */ mint, 
        /* token_program */ token_program.address(),
        /* deposit signer */ &Signer::from(&deposit_signer_seeds)
    )?;

    //      Transfer from vault to deposit
    let vault_bump = &[vault_pda.1]; // Prevent dropping
    let vault_signer_seed = VaultData::vault_signer_seeds(
        authority.address(), 
        &vault_index_bytes, 
        mint.address(), 
        token_program.address(), 
        vault_bump);
    Transfer {
        from: vault,
        to: deposit,
        authority: vault,
        amount
    }.invoke_signed(&[Signer::from(&vault_signer_seed)])?;


    // Decrement open transfers from the vault data.
    // SAFETY: Only mutable here. Vault data bytes are a valid representation of VaultData
    let vault_data_mut = unsafe { &mut *(vault_data.data_ptr() as *mut VaultData) };
    vault_data_mut.set_open_transfers(vault_data_mut.open_transfers() + 1);

    ProgramResult::Ok(())
}
