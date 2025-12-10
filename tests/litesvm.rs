#[cfg(test)]
mod litesvm_tests {
    use std::env::{current_dir, home_dir};
    use std::path::Path;

    use litesvm::LiteSVM;
    use pime::interface::pime_instruction::PimeInstruction;
    use pime::states::Vault;
    use solana_sdk::message::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey, rent, signature::Keypair, signer::Signer, transaction::Transaction};
    use spl_token_interface::state::Mint;
    use spl_token_interface::state::Account as TokenAccount;

    #[test]
    fn create_vault() {
        let mut svm = LiteSVM::new();

        let pime_id = Pubkey::new_from_array(pime::ID);
        let pime_program_path = Path::join(&current_dir().unwrap(), "target/sbpf-solana-solana/release/pime.so");
        if let Err(e) = svm.add_program_from_file(pime_id, pime_program_path) {
            panic!("Could not add Pime program: {}", e);
        }

        let token_program = spl_token_interface::ID;
        // let native_mint = spl_token_interface::native_mint::ID;

        let from_keypair = Keypair::new();
        let from = from_keypair.pubkey();
        let to = Pubkey::new_unique();
        
        svm.airdrop(&from, LAMPORTS_PER_SOL).unwrap();

        let mint = create_and_mint_to(&mut svm, &from_keypair, &to, 100_000, &token_program).unwrap();

        let authority = Keypair::new();
        svm.airdrop(&authority.pubkey(), LAMPORTS_PER_SOL).unwrap();

        let index = 0u64;
        let vault_data = find_vault_data_pda(&authority.pubkey(), index, &mint, &token_program);
        let vault = find_vault_pda(&vault_data.0);

        let create_vault_inst = Instruction::new_with_bytes(
            /* program id*/ pime_id, 
            /* data */ PimeInstruction::serialize_create_vault_inst_data(
                index, 
                /* timeframe */ 0, 
                /* max_withdraws */ 0, 
                /* max_lamport_withdraw */ 0
            ).as_slice(), 
            /* accounts */ [
                AccountMeta::new(authority.pubkey(), true),
                AccountMeta::new(vault_data.0, false),
                AccountMeta::new(vault.0, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            ].to_vec()
        );

        let r = svm.send_transaction(Transaction::new(
            /* from keypairs */ &[&authority], 
            /* message */ Message::new(
                /* instructions */ &[create_vault_inst],
                /* payer */ Some(&authority.pubkey())),
            /* latest blockhash */ svm.latest_blockhash()
        ));
        panic!("{:#?}", r);
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

    fn find_vault_data_pda(authority: &Pubkey, index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[
            Vault::VAULT_DATA_SEED,
            &authority.to_bytes(),
            &index.to_le_bytes(),
            &mint.to_bytes(),
            &token_program.to_bytes(),
        ], 
            &Pubkey::new_from_array(pime::ID))
    }

    fn find_vault_pda(vault_data: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[
            &vault_data.to_bytes(),
        ],
            &Pubkey::new_from_array(pime::ID))
    }
}
