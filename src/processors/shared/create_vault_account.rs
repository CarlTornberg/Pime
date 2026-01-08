use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, pubkey::Pubkey};
use pinocchio_token::state::TokenAccount;

/// Create the Vault Token Account.
///
/// Will fail if account exists (Does not check)
pub fn create_vault_account (
    payer: &AccountInfo, 
    vault: &AccountInfo,
    mint: &AccountInfo, 
    token_program: &Pubkey, 
    vault_signer: &Signer,
) -> ProgramResult {
    let signer = core::slice::from_ref(vault_signer);

    pinocchio_system::create_account_with_minimum_balance_signed(
            /* account */ vault, 
            /* space */ TokenAccount::LEN,
            /* owner */ token_program, 
            /* payer */ payer, 
            /* rent sysvar */ None,
            /* signers */ signer,
        )?;

    pinocchio_token::instructions::InitializeAccount3 {
        account: vault,
        mint,
        owner: vault.key(),
    }.invoke_signed(signer)?;

    Ok(())
}
