use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, sysvars::clock::UnixTimestamp};

use crate::states::{Transmutable, VaultData, VaultHistory, as_bytes};

pub(crate) fn process_create_vault_data_account(
    authority: &AccountInfo, 
    vault_data: &AccountInfo, 
    max_transactions: u64, 
    timeframe: i64, 
    max_lamports: u64, 
    transfer_min_warmup: UnixTimestamp,
    transfer_max_window: UnixTimestamp,
    vault_data_signer: &Signer) -> Result<(), ProgramError> {
    let signer = core::slice::from_ref(vault_data_signer);

    let vault_data_size = size_of::<VaultData>() + (max_transactions as usize * size_of::<VaultHistory>());
    pinocchio_system::
        create_account_with_minimum_balance_signed(
            /* account */ vault_data, 
            /* space */ vault_data_size, 
            /* owner */ &crate::ID, 
            /* payer */ authority, 
            /* rent sysvar */ None,
            /* signer seeds */ signer
        )?;

    // SAFETY: Data is not previously borrowed and is represented by a valid format.
    let vault_data_mut = unsafe {
        core::slice::from_raw_parts_mut(
            vault_data.data_ptr(), 
            size_of::<VaultData>())
    };
    vault_data_mut.copy_from_slice(as_bytes(
        &VaultData::new(
            *authority.key(), 
            timeframe, 
            max_lamports, 
            max_transactions,
            transfer_min_warmup,
            transfer_max_window,
        )));

    let h = VaultHistory::new(UnixTimestamp::MIN, u64::MIN);
    let fake_history = as_bytes(&h);
    for i in 0..(max_transactions as usize) {
        unsafe {
            core::slice::from_raw_parts_mut(
                vault_data.data_ptr().add(VaultData::LEN + i * VaultHistory::LEN), 
                VaultHistory::LEN)
                .copy_from_slice(fake_history);
        }
    }

    Ok(())
}
