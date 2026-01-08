use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::Signer, pubkey::Pubkey};
use pinocchio_token::state::TokenAccount;


pub fn create_deposit_account (
    payer: &AccountInfo,
    deposit: &AccountInfo, 
    mint: &AccountInfo,
    token_program: &Pubkey,
    deposit_signer: &Signer,
) -> ProgramResult {

    let signer = core::slice::from_ref(deposit_signer);
    pinocchio_system::create_account_with_minimum_balance_signed(
            /* account */ deposit, 
            /* space */ TokenAccount::LEN,
            /* owner */ token_program, 
            /* payer */ payer, 
            /* rent sysvar */ None,
            /* signers */ signer,
        )?;

    pinocchio_token::instructions::InitializeAccount3 {
        account: deposit,
        mint,
        owner: deposit.key(),
    }.invoke_signed(signer)?;

    Ok(())
}
