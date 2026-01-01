use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, msg, program_error::ProgramError, pubkey::pubkey_eq, seeds};

use crate::{errors::PimeError, states::{VaultData, from_bytes, transfer_data::TransferData}};

/// Transfers assets from its booked vault to the received.
pub fn execute_transfer(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {
    
    //   - `u64`       The vault index.
    //   - `u64`       The transfer index.
    
    //      Deserialize instruction data
    let (vault_index, transfer_index) = (1,2);


    //   0. `[signer]`     The owner of the vault.
    //   1. `[writeable]`  The vault account.
    //   2. `[writeable]`  The transfer account.
    //   3. `[writeable]`  The deposit account.
    //   4. `[writeable]`  The destination account.
    //   4. `[]`           The mint address of the vault/transfer. 
    //   5. `[]`           The token program. 
    let [authority, vault, transfer, deposit, destination, mint, token_program, _remaining @ ..] = accounts else {
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
    if !vault.is_owned_by(&crate::ID) {
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
        // TODO create token account.
        msg!("Destination account is not created. Ask the owner to create the account. TODO Initialize account.");
        return Err(ProgramError::UninitializedAccount);
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














