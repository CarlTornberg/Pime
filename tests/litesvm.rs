#[cfg(test)]
mod litesvm_tests {
    use litesvm::LiteSVM;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey, rent, signature::Keypair, signer::Signer, transaction::Transaction};
    use spl_token_interface::state::{GenericTokenAccount, Mint};
    use spl_token_interface::state::Account as TokenAccount;

    #[test]
    fn create_vault() {
        let mut svm = LiteSVM::new();

        let pime_id = Pubkey::new_from_array(pime::ID);
        let token_program = spl_token_interface::ID;
        // let native_mint = spl_token_interface::native_mint::ID;

        let from_keypair = Keypair::new();
        let from = from_keypair.pubkey();
        let to = Pubkey::new_unique();
        
        svm.airdrop(&from, LAMPORTS_PER_SOL).unwrap();

        create_and_mint_to(&mut svm, &from_keypair, &to, 100_000, &token_program).unwrap();

    }

    // Helpers 
    
    /// Creates a new mint, and mints the token to the to account.
    /// This function initializes the ATA of the to account.
    ///
    /// Returns the new mint address.
    fn create_and_mint_to(svm: &mut LiteSVM, mint_authority: &Keypair, to: &Pubkey, mint_amount: u64, token_program: &Pubkey) -> Result<Pubkey, ProgramError> {
        let mint = Keypair::new();
        let to_ata = Pubkey::find_program_address(
            &[
                &to.to_bytes(),
                &token_program.to_bytes(),
                &mint.pubkey().to_bytes(),
            ], 
            &spl_associated_token_account_interface::program::ID);

        let create_mint_account_inst = solana_system_interface::instruction::create_account(
            /* from */ &mint_authority.pubkey(), 
            /* to */ &mint.pubkey(),
            /* lamports */ svm.minimum_balance_for_rent_exemption(Mint::LEN),
            /* space */ Mint::LEN as u64,
            /* owner */ token_program,
        );

        let init_mint_inst = spl_token_interface::instruction::initialize_mint(
            /* token program */ token_program, 
            /* mint */ &mint.pubkey(), 
            /* mint authority */ &mint_authority.pubkey(), 
            /* freeze authority */ None, 
            /* decimals */ 6,
        )?;

        let create_ata_inst = spl_associated_token_account_interface::instruction::create_associated_token_account(
            /* funding address */ &mint_authority.pubkey(),
            /* wallet_address */ to, 
            /* token_mint_address */ &mint.pubkey(), 
            /* token_program_id */ token_program);

        let mint_to_inst = spl_token_interface::instruction::mint_to(
            /* token program */ token_program,  
            /* mint */ &mint.pubkey(), 
            /* account */ &to_ata.0, 
            /* owner */ &mint_authority.pubkey(), 
            /* signer pubkeys */ &[&mint_authority.pubkey()], 
            /* amount */ mint_amount 
        )?;

        let tx = Transaction::new(
            /* Signers */ &[mint_authority, &mint], 
            Message::new(
                &[
                    create_mint_account_inst,
                    init_mint_inst,
                    create_ata_inst,
                    mint_to_inst,
                ],
                /* Payer */ Some(&mint_authority.pubkey())
            ), 
            svm.latest_blockhash()
        );
        let _ = svm.send_transaction(tx).unwrap();
        let to_ata_account = svm.get_account(&to_ata.0).unwrap();
        let to_ata_token_account = TokenAccount::unpack(&to_ata_account.data).unwrap();
        assert_eq!(to_ata_token_account.amount, mint_amount);
        Ok(mint.pubkey())
    }
}
