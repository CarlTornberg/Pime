use pinocchio::{AccountView, Address, ProgramResult, address::address_eq, cpi::Signer, error::ProgramError, sysvars::{Sysvar, clock::Clock}};
use solana_program_log::log;

use crate::{errors::PimeError, interface::instructions::execute_transfer::ExecuteTransferInstructionData, states::{VaultData, from_bytes, transfer_data::TransferData}};

/// Transfers assets from its booked vault to the received.
pub fn execute_transfer(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    
    //      Deserialize instruction data
    if instruction_data.len() < size_of::<ExecuteTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let (vault_index, transfer_index) = (
        u64::from_le_bytes(unsafe {*(instruction_data.as_ptr() as *const [u8; size_of::<u64>()])}),
        u64::from_le_bytes(unsafe {*(instruction_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()])})
    );

    let [authority, vault_data, transfer, deposit, destination, mint, token_program, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_data_pda = VaultData::find_vault_data_address(authority.address(), vault_index, mint.address(), token_program.address());
    if !address_eq(vault_data.address(), &vault_data_pda.0) {
        log!("Invalid Vault Data PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault_data.lamports() == 0 {
        log!("Vault data is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault_data.owned_by(&crate::ID) {
        log!("Vault data has illegal owner.");
        return Err(ProgramError::IllegalOwner);
    }
    if !vault_data.is_writable() {
        log!("Vault data is not mutable.");
        return Err(ProgramError::Immutable);
    }

    let transfer_pda = TransferData::find_transfer_address(authority.address(), destination.address(), vault_index, transfer_index, mint.address(), token_program.address());
    if !address_eq(transfer.address(), &transfer_pda.0) {
        log!("Incorrect Transfer PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() == 0 {
        log!("Transfer is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    if !transfer.owned_by(&crate::ID) {
        log!("Transfer is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    //      Check that target account is the account the deposit should go to
    if transfer.data_len() > size_of::<TransferData>() {
        log!("Transfer has invalid account data.");
        return Err(ProgramError::AccountDataTooSmall);
    }

    let deposit_pda = TransferData::find_deposit_address(authority.address(), destination.address(), vault_index, transfer_index, mint.address(), token_program.address());
    if !address_eq(deposit.address(), &deposit_pda.0) {
        log!("Incorrect Deposit PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() == 0 {
        log!("Deposit it not created. Has a transfer been booked?");
        return Err(ProgramError::UninitializedAccount);
    }
    if !deposit.owned_by(token_program.address()) {
        log!("Deposit account is not owned by the supplied token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if destination.lamports() == 0 {
        let [system_program, ata_owner, a_token, _remainder @ .. ] = remaining else {
            log!("Requires system program, ata owner, and associated token program.");
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !address_eq(a_token.address(), &pinocchio_associated_token_account::ID) {
            log!("Associated token program is incorrect.");
            return Err(ProgramError::IllegalOwner);
        }
        let ata = Address::find_program_address(&[
            ata_owner.address().as_array(),
            token_program.address().as_array(),
            mint.address().as_array(),
        ], a_token.address());
        if !address_eq(destination.address(), &ata.0) {
            log!("Destination ATA is not derived from the provided owner.");
            return Err(PimeError::DestinationMismatch.into());
        }
        pinocchio_associated_token_account::instructions::Create{
            funding_account: authority,
            account: destination,
            wallet: ata_owner,
            mint,
            system_program,
            token_program
        }.invoke()?;
    }
    if !destination.owned_by(token_program.address()) {
        log!("Destination account is not owned by the supplied token program");
        return Err(ProgramError::InvalidAccountOwner);
    }
    
    if !mint.owned_by(token_program.address()) {
        log!("Mint now owned by the supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      Data safety checks
    // SAFETY data is not borrowed earlier and of type Transmutable
    let transfer_data = unsafe { from_bytes::<TransferData>(transfer.borrow_unchecked())? } ;
    if !address_eq(&transfer_data.destination, destination.address()) {
        log!("Supplied destination account does not match expected account");
        return Err(PimeError::DestinationMismatch.into());
    }
    let now = Clock::get()?.unix_timestamp;
    if now < transfer_data.created() + transfer_data.warmup() {
        log!("Warm-up period has not yet passed.");
        return Err(PimeError::TransferWarmingUp.into());
    }
    if now > transfer_data.created() + transfer_data.validity() {
        log!("Transfer has expired. Close this transfer and create a new one.");
        return Err(PimeError::TransferExpired.into());
    }
    //      Transfer from deposit to target account
    let vault_index_bytes = vault_index.to_le_bytes();
    let transfer_index_bytes = transfer_index.to_le_bytes();
    let deposit_bump = &[deposit_pda.1];
    let deposit_signer_seeds = TransferData::deposit_signer_seeds(
        authority.address(), 
        destination.address(),
        &vault_index_bytes, 
        &transfer_index_bytes, 
        mint.address(), 
        token_program.address(), 
        deposit_bump
    );
    pinocchio_token::instructions::Transfer {
        from: deposit,
        to: destination,
        authority: deposit,
        amount: transfer_data.amount(),
    }.invoke_signed(&[Signer::from(&deposit_signer_seeds)])?;

    //      Close deposit
    pinocchio_token::instructions::CloseAccount {
        account: deposit,
        destination: authority,
        authority: deposit,
    }.invoke_signed(&[Signer::from(&deposit_signer_seeds)])?;

    //      Close transfer
    unsafe {
        // Moves the lamports to the destination account.
        //
        // Note: This is safe since the runtime checks for balanced instructions
        // before and after each CPI and instruction, and the total lamports
        // supply is bound to `u64::MAX`.
        authority.set_lamports(authority.lamports() + transfer.lamports());
        transfer.close_unchecked();
    };

    // Decrement open transfers from the vault data.
    // SAFETY: Only mutable here. Vault data bytes are a valid representation of VaultData
    let vault_data_mut = unsafe { &mut *(vault_data.data_ptr() as *mut VaultData) };
    vault_data_mut.set_open_transfers(vault_data_mut.open_transfers() - 1);
    
    ProgramResult::Ok(())
}














