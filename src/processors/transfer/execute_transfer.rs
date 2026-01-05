use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::{find_program_address, pubkey_eq}, seeds, sysvars::{Sysvar, clock::Clock}};

use crate::{errors::PimeError, interface::instructions::execute_transfer::ExecuteTransferInstructionData, states::{VaultData, from_bytes, transfer_data::TransferData}};

/// Transfers assets from its booked vault to the received.
pub fn execute_transfer(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {
    
    //      Deserialize instruction data
    if instrution_data.len() < size_of::<ExecuteTransferInstructionData>() - size_of::<u8>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let (vault_index, transfer_index) = (
        u64::from_le_bytes(unsafe {*(instrution_data.as_ptr() as *const [u8; size_of::<u64>()])}),
        u64::from_le_bytes(unsafe {*(instrution_data.as_ptr().add(size_of::<u64>()) as *const [u8; size_of::<u64>()])})
    );

    let [authority, vault, transfer, deposit, destination, mint, token_program, remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let vault_data_pda = VaultData::get_vault_data_pda(authority.key(), vault_index, mint.key(), token_program.key());
    let vault_pda = VaultData::get_vault_pda(&vault_data_pda.0);
    if !pubkey_eq(vault.key(), &vault_pda.0) {
        msg!("Invalid Vauld PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if vault.lamports() == 0 {
        msg!("Vault is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    if !vault.is_owned_by(token_program.key()) {
        msg!("Vault has illegal owner.");
        return Err(ProgramError::IllegalOwner);
    }

    let transfer_pda = TransferData::get_transfer_pda(&vault_data_pda.0, &transfer_index);
    if !pubkey_eq(transfer.key(), &transfer_pda.0) {
        msg!("Incorrect Transfer PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if transfer.lamports() == 0 {
        msg!("Transfer is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }
    if !transfer.is_owned_by(&crate::ID) {
        msg!("Transfer is not owned by this program.");
        return Err(ProgramError::IllegalOwner);
    }
    //      Check that target account is the account the deposit should go to
    if transfer.data_len() > size_of::<TransferData>() {
        msg!("Transfer has invalid account data.");
        return Err(ProgramError::AccountDataTooSmall);
    }

    let deposit_pda = TransferData::get_deposit_pda(&transfer_pda.0);
    if !pubkey_eq(deposit.key(), &deposit_pda.0) {
        msg!("Incorrect Deposit PDA");
        return Err(PimeError::IncorrectPDA.into());
    }
    if deposit.lamports() == 0 {
        msg!("Deposit it not created. Has a transfer been booked?");
        return Err(ProgramError::UninitializedAccount);
    }
    if !deposit.is_owned_by(token_program.key()) {
        msg!("Deposit account is not owned by the supplied token program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if destination.lamports() == 0 {
        let [system_program, ata_owner, a_token, _remainder @ .. ] = remaining else {
            msg!("Requires system program, ata owner, and associated token program.");
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !pubkey_eq(a_token.key(), &pinocchio_associated_token_account::ID) {
            msg!("Associated token program is incorrect.");
            return Err(ProgramError::IllegalOwner);
        }
        let ata = find_program_address(&[
            ata_owner.key(),
            token_program.key(),
            mint.key(),
        ], a_token.key());
        if !pubkey_eq(destination.key(), &ata.0) {
            msg!("Destination ATA is not derived from the provided owner.");
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
    if !destination.is_owned_by(token_program.key()) {
        msg!("Destination account is not owned by the supplied token program");
        return Err(ProgramError::InvalidAccountOwner);
    }
    
    if !mint.is_owned_by(token_program.key()) {
        msg!("Mint now owned by the supplied token program.");
        return Err(ProgramError::InvalidAccountOwner);
    }

    //      Data safety checks
    // SAFETY data is not borrowed earlier and of type Transmutable
    let transfer_data = unsafe { from_bytes::<TransferData>(transfer.borrow_data_unchecked())? } ;
    if !pubkey_eq(&transfer_data.destination, destination.key()) {
        msg!("Supplied destination account does not match expected account");
        return Err(PimeError::DestinationMismatch.into());
    }
    let now = Clock::get()?.unix_timestamp;
    if now < transfer_data.created() + transfer_data.warmup() {
        msg!("Warm-up period has not yet passed.");
        return Err(PimeError::TransferWarmingUp.into());
    }
    if now > transfer_data.created() + transfer_data.validity() {
        msg!("Transfer has expired. Close this transfer and create a new one.");
        return Err(PimeError::TransferExpired.into());
    }
    //      Transfer from deposit to target account
    let deposit_bump = &[deposit_pda.1];
    let deposit_signer_seeds = seeds!(transfer.key(), deposit_bump);
    pinocchio_token::instructions::Transfer {
        from: deposit,
        to: destination,
        authority: deposit,
        amount: transfer_data.amount(),
    }.invoke_signed(&[Signer::from(&deposit_signer_seeds)])?;

    //      Close deposit
    pinocchio_token::instructions::CloseAccount {
        account: deposit,
        destination: vault,
        authority: deposit,
    }.invoke_signed(&[Signer::from(&deposit_signer_seeds)])?;

    //      Close transfer
    unsafe {
        // Moves the lamports to the destination account.
        //
        // Note: This is safe since the runtime checks for balanced instructions
        // before and after each CPI and instruction, and the total lamports
        // supply is bound to `u64::MAX`.
        *authority.borrow_mut_lamports_unchecked() += transfer.lamports();
        transfer.close_unchecked();
    };
    
    ProgramResult::Ok(())
}














