use pinocchio::{ProgramResult, account_info::AccountInfo, instruction::{Seed, Signer}, pubkey::Pubkey, seeds};
use pinocchio_token::state::TokenAccount;


pub fn create_deposit_account (
    payer: &AccountInfo,
    deposit: &AccountInfo, 
    transfer: &AccountInfo, 
    deposit_bump: &[u8],
    mint: &AccountInfo,
    token_program: &Pubkey,
) -> ProgramResult {
    let seed = seeds!(transfer.key(), deposit_bump);

    pinocchio_system::create_account_with_minimum_balance_signed(
            /* account */ deposit, 
            /* space */ TokenAccount::LEN,
            /* owner */ token_program, 
            /* payer */ payer, 
            /* rent sysvar */ None,
            /* signers */ &[Signer::from(&seed)],
        )?;

    pinocchio_token::instructions::InitializeAccount3 {
        account: deposit,
        mint,
        owner: deposit.key(),
    }.invoke_signed(&[Signer::from(&seed)])?;

    Ok(())
}
