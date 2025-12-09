#[cfg(test)]
mod litesvm_tests {
    use litesvm::LiteSVM;
    use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey, rent, signature::Keypair, signer::Signer, transaction::Transaction};
    use spl_token_interface::state::Mint;

    #[test]
    fn create_vault() {
        let mut svm = LiteSVM::new();

        let pime_id = Pubkey::new_from_array(pime::ID);

        let from_keypair = Keypair::new();
        let from = from_keypair.pubkey();
        let to = Pubkey::new_unique();

        let token_program = spl_token_interface::ID;
        let native_mint = spl_token_interface::native_mint::ID;
        let mint = Keypair::new();

        let mint_space = Mint::LEN;
        let create_mint_account_inst = solana_system_interface::instruction::create_account(
            /* from */ &from, 
            /* to */ &mint.pubkey(),
            /* lamports */ svm.minimum_balance_for_rent_exemption(mint_space),
            /* space */ mint_space as u64,
            /* owner */ &token_program,
        );

        let init_mint_inst = spl_token_interface::instruction::initialize_mint(
            /* token program */ &token_program, 
            /* mint */ &mint.pubkey(), 
            /* mint authority */ &from, 
            /* freeze authority */ None, 
            /* decimals */ 6,
        ).unwrap();

        let to_ata = Pubkey::find_program_address(
            &[
                &to.to_bytes(),
                &token_program.to_bytes(),
                &mint.pubkey().to_bytes(),
            ], 
            &spl_associated_token_account_interface::program::ID);

        let create_ata_inst = spl_associated_token_account_interface::instruction::create_associated_token_account(
            /* funding address*/ &from,
            /* wallet_address */ &from, 
            /* token_mint_address */ &mint.pubkey(), 
            /* token_program_id */ &token_program);

        let mint_to_inst = spl_token_interface::instruction::mint_to(
            &token_program, /* token program */ 
            &mint.pubkey(), /* mint */
            &to_ata.0, /* account */
            &from, /* owner */
            &[&from], /* signer pubkeys */
            67 /* amount */
        ).unwrap();

        svm.airdrop(&from, LAMPORTS_PER_SOL).unwrap();

        let tx = Transaction::new(
            &[&from_keypair, &mint], 
            Message::new(
                &[
                    create_mint_account_inst,
                    init_mint_inst,
                    create_ata_inst,
                    mint_to_inst,
                ],
                Some(&from)
            ), 
            svm.latest_blockhash()
        );
        let tx_res = svm.send_transaction(tx);
        match tx_res {
            Ok(res) => panic!("success: {:#?}", res),
            Err(f) => panic!("failed: {:#?}", f)
        }


        let from_account = svm.get_account(&from).unwrap();
        assert_ne!(from_account.lamports, LAMPORTS_PER_SOL);


    }

}
