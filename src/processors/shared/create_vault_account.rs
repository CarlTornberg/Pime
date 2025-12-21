use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, pubkey::Pubkey, seeds};
use pinocchio_token::state::TokenAccount;

/// Create the Vault Token Account.
///
/// Will fail if account exists (Does not check)
pub fn create_vault_account (
    payer: &AccountInfo, 
    vault: &AccountInfo, 
    vault_bump: u8,
    vault_data_pubkey: &Pubkey, 
    mint: &AccountInfo, 
    token_program: &Pubkey, 
) -> ProgramResult {
    let vault_bump = &[vault_bump];
    let vault_signer_seeds = seeds!( vault_data_pubkey, vault_bump );

    pinocchio_system::create_account_with_minimum_balance_signed(
            /* account */ vault, 
            /* space */ TokenAccount::LEN,
            /* owner */ token_program, 
            /* payer */ payer, 
            /* rent sysvar */ None,
            /* signers */ &[Signer::from(&vault_signer_seeds)],
        )?;

    pinocchio_token::instructions::InitializeAccount3 {
        account: vault,
        mint,
        owner: vault.key(),
    }.invoke_signed(&[Signer::from(&vault_signer_seeds)])?;

    Ok(())
}
