use pinocchio::{account_info::AccountInfo, program_error::ProgramError, sysvars::{Sysvar, clock::Clock}};

use crate::{errors::PimeError, states::{VaultData, VaultHistory, as_bytes}};

///
/// Check if a vault can withdraw given its history
///
/// # SAFETY
/// The byte array "bytes" must be a valid array representation (n>0) of VaultHistory
pub unsafe fn withdraw(vault: &AccountInfo, to: &AccountInfo, amount: u64, mint: &AccountInfo, token_program: &AccountInfo, vault_data: &mut VaultData, vault_bump: u8, bytes: &mut [u8]) -> Result<(), ProgramError> {
    let max_transactions = vault_data.max_transactions() as usize;
    let max_lamports = vault_data.max_lamports();
    let mut i = vault_data.transaction_index() as usize;
    let timeframe = vault_data.timeframe();

    let now = Clock::get()?.unix_timestamp;

    let mut vault_history;
    let mut tot_timeframe_lamports = 0u64;

    // Loop all transactions in the byte array
    for _ in 0 .. max_transactions {
        i += 1;
        if i == max_transactions { i = 0; }

        vault_history = &*(bytes.as_ptr().add(i * size_of::<VaultHistory>()) as *const VaultHistory);

        if vault_history.timestamp() + timeframe < now {
            tot_timeframe_lamports += vault_history.amount();
        }
        else { 
            // Check if the amount can be withdrawn in this timeframe
            if tot_timeframe_lamports + amount > max_lamports {
                return Err(PimeError::WithdrawLimitReachedAmount.into());
            }
            withdraw_unchecked(vault, to, amount, mint, token_program, vault_bump)?;

            core::slice::from_raw_parts_mut(
                bytes.as_mut_ptr().add(i * size_of::<VaultHistory>()), 
                size_of::<VaultHistory>())
                .copy_from_slice(as_bytes(&VaultHistory::new(now, amount)));


            return Ok(());
        }
    }


    Ok(())
}

pub fn withdraw_unchecked(vault: &AccountInfo, to: &AccountInfo, amount: u64, mint: &AccountInfo, token_program: &AccountInfo, vault_bump: u8) -> Result<(), ProgramError> {
    Ok(())
}
