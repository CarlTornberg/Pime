#[cfg(test)]
mod litesvm_tests {
    use std::env::current_dir;
    use std::path::Path;

    use litesvm::LiteSVM;
    use pime::interface::instructions::book_transfer::BookTransferInstructionData;
    use pime::interface::instructions::create_vault_instruction::CreateVaultInstructionData;
    use pime::interface::instructions::deposit_to_vault_instruction::DepositToVaultInstructionData;
    use pime::interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData;
    use pime::shared;
    use pime::states::{VaultData, VaultHistory, as_bytes};
    use solana_sdk::message::{AccountMeta, Instruction};
    use solana_sdk::{message::Message, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
    use spl_associated_token_account_interface::address::{get_associated_token_address, get_associated_token_address_with_program_id};
    use spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent;
    use spl_token_interface::state::Mint;
    use spl_token_interface::state::Account as TokenAccount;

    const PIME_ID: Pubkey = Pubkey::new_from_array(pime::ID);
    const TOKEN_PROGRAM: Pubkey = spl_token_interface::ID;

    #[test]
    fn alice_creates_vault() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();

        let create_vault_instruction_data = CreateVaultInstructionData::new(
            /* index */ 0u64, 
            /* timeframe */ 1i64, 
            /* max_withdraws */ 2u64, 
            /* max_lamports */ 3u64,
            /* transfer min window */ 4u64,
            /* transfer max_window */ 5u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(&mut svm, &alice.pubkey(), &mint, &alice);

        // Create vault based on the new mint
        create_new_vault(&mut svm, &alice, &create_vault_instruction_data, &mint.pubkey());

    }

    #[test]
    fn alice_deposits_to_vault() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();

        let create_vault_instruction_data = CreateVaultInstructionData::new(
            /* index */ 1u64, 
            /* timeframe */ 2i64, 
            /* max_withdraws */ 3u64, 
            /* max_lamports */ 4u64,
            /* transfer min window */ 4u64,
            /* transfer max_window */ 5u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(
            /* svm */ &mut svm, 
            /* authority */ &alice.pubkey(), 
            /* mint */ &mint, 
            /* payer */ &alice);

        // Create vault based on the new mint
        create_new_vault(
            /* svm */ &mut svm, 
            /* authority */ &alice, 
            /* instruction data */ &create_vault_instruction_data, 
            /* mint */ &mint.pubkey()
        );

        let alice_ata = get_associated_token_address_with_program_id (
            &alice.pubkey(), 
            &mint.pubkey(), 
            &TOKEN_PROGRAM);
        let mint_amount = 10_000;
        mint_to(
            /* svm */ &mut svm,
            /* mint */ &mint.pubkey(), 
            /* mint authority */ &alice, 
            /* to */ &alice.pubkey(),
            /* to ata */ &alice_ata, 
            /* payer */ &alice, 
            /* amount */ mint_amount);

        let deposit_amount = 4_000;
        let deposit_inst = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* index */ create_vault_instruction_data.index(), 
            /* deposit amount */ deposit_amount);

        deposit_to_vault(
            &mut svm, 
            /* from */ &alice_ata, 
            /* from_authority */ &alice, 
            &mint.pubkey(), 
            &deposit_inst);
    }

    #[test]
    fn alice_withdraws_from_own_vault() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();

        let create_vault_instruction_data = CreateVaultInstructionData::new(
            /* index */ 1u64, 
            /* timeframe */ 2i64, 
            /* max_withdraws */ 3u64, 
            /* max_lamports */ 4u64,
            /* transfer min window */ 5u64,
            /* transfer max_window */ 6u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(
            /* svm */ &mut svm, 
            /* authority */ &alice.pubkey(), 
            /* mint */ &mint, 
            /* payer */ &alice);

        // Create vault based on the new mint
        create_new_vault(
            /* svm */ &mut svm, 
            /* authority */ &alice, 
            /* instruction data */ &create_vault_instruction_data, 
            /* mint */ &mint.pubkey()
        );

        let alice_ata = get_associated_token_address_with_program_id (
            /* wallet address */ &alice.pubkey(), 
            /* mint */ &mint.pubkey(), 
            /* token program */ &TOKEN_PROGRAM);
        let mint_amount = 10_000;
        mint_to(
            /* svm */ &mut svm,
            /* mint */ &mint.pubkey(), 
            /* mint authority */ &alice, 
            /* to */ &alice.pubkey(),
            /* to ata */ &alice_ata, 
            /* payer */ &alice, 
            /* amount */ mint_amount);

        let deposit_amount = 4_000;
        let deposit_inst = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* index */ create_vault_instruction_data.index(), 
            /* deposit amount */ deposit_amount);

        deposit_to_vault(
            &mut svm, 
            /* from */ &alice_ata, 
            /* from_authority */ &alice, 
            &mint.pubkey(), 
            &deposit_inst);

        let withdraw_inst = WithdrawFromVaultInstructionData::new(
            /* amount */ 1, 
            /* vault index */ create_vault_instruction_data.index());

        withdraw_from_vault(
            /* svm */ &mut svm,
            /* vault_authority */ &alice,
            /* to */ &alice_ata,
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM, 
            /* inst */ &withdraw_inst,
        );
    }

    #[test]
    fn alice_books_transfer() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();
        let mint = Keypair::new();
        let alice_ata = get_associated_token_address(&alice.pubkey(), &mint.pubkey());

        let receiver = Keypair::new();
        let receiver_ata = get_associated_token_address(&receiver.pubkey(), &mint.pubkey());

        let mint_amount = 1_000;

        // Mint tokens
        initialize_mint(
            /* svm */ &mut svm, 
            /* authority */ &alice.pubkey(), 
            /* mint */ &mint, 
            /* payer */ &alice);
        mint_to(&mut svm, 
            /* mint */ &mint.pubkey(), 
            /* mint_authority */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* payer */ &alice, 
            /* amount */ mint_amount);

        // Create vault
        let create_vault_inst_data = CreateVaultInstructionData::new(
            /* index */ 1, 
            /* timeframe */ 2, 
            /* max_transactions */ 3, 
            /* max_amount */ 4, 
            /* transfer_min_warmup */ 5, 
            /* transfer_max_window */ 6);
        create_new_vault(&mut svm, 
            /* authority */ &alice, 
            /* instuction_data */ &create_vault_inst_data,  
            /* mint */ &mint.pubkey()
        );

        // Deposit to vault
        let deposit_amount = 500;
        let deposit_to_vault_inst_data = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* index */ create_vault_inst_data.index(), 
            /* amount */ deposit_amount);
        deposit_to_vault(&mut svm, 
            /* from_acc */ &alice_ata, 
            /* from_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* inst */ &deposit_to_vault_inst_data);

        // Book transfer
        let transfer_amount = 250;
        let book_transfer_inst_data = BookTransferInstructionData::new(
            /* amount */ transfer_amount, 
            /* destination */ receiver_ata.to_bytes(),
            /* vault_index */ create_vault_inst_data.index(), 
            /* transfer_index */ 1, 
            /* warmup */ 1, 
            /* validity*/ 1);

        book_transfer(&mut svm, 
            /* inst data */ &book_transfer_inst_data, 
            /* authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);

        // Assert vault and deposit
        let vault_data = find_vault_data_pda(&alice.pubkey(), create_vault_inst_data.index(), &mint.pubkey(), &TOKEN_PROGRAM);
        let vault = find_vault_pda(&vault_data.0);
        let transfer = find_transfer_pda(&vault_data.0, book_transfer_inst_data.transfer_index());
        let deposit = find_deposit_pda(&transfer.0);
        let vault_acc = TokenAccount::unpack(&svm.get_account(&vault.0).unwrap().data).unwrap();
        let deposit_acc = TokenAccount::unpack(&svm.get_account(&deposit.0).unwrap().data).unwrap();

        assert_eq!(vault_acc.amount, deposit_amount - transfer_amount);
        assert_eq!(deposit_acc.amount, transfer_amount);

    }

    #[test]
    fn alice_executes_transfer() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();
        let mint = Keypair::new();
        let alice_ata = get_associated_token_address(&alice.pubkey(), &mint.pubkey());

        let receiver = Keypair::new();
        let receiver_ata = get_associated_token_address(&receiver.pubkey(), &mint.pubkey());

        let mint_amount = 1_000;

        // Mint tokens
        initialize_mint(
            /* svm */ &mut svm, 
            /* authority */ &alice.pubkey(), 
            /* mint */ &mint, 
            /* payer */ &alice);
        mint_to(&mut svm, 
            /* mint */ &mint.pubkey(), 
            /* mint_authority */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* payer */ &alice, 
            /* amount */ mint_amount);

        // Create vault
        let create_vault_inst_data = CreateVaultInstructionData::new(
            /* index */ 1, 
            /* timeframe */ 2, 
            /* max_transactions */ 3, 
            /* max_amount */ 4, 
            /* transfer_min_warmup */ 5, 
            /* transfer_max_window */ 6);
        create_new_vault(&mut svm, 
            /* authority */ &alice, 
            /* instuction_data */ &create_vault_inst_data,  
            /* mint */ &mint.pubkey()
        );

        // Deposit to vault
        let deposit_amount = 500;
        let deposit_to_vault_inst_data = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* index */ create_vault_inst_data.index(), 
            /* amount */ deposit_amount);
        deposit_to_vault(&mut svm, 
            /* from_acc */ &alice_ata, 
            /* from_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* inst */ &deposit_to_vault_inst_data);

        // Book transfer
        let transfer_amount = 250;
        let book_transfer_inst_data = BookTransferInstructionData::new(
            /* amount */ transfer_amount, 
            /* destination */ receiver_ata.to_bytes(),
            /* vault_index */ create_vault_inst_data.index(), 
            /* transfer_index */ 1, 
            /* warmup */ 1, 
            /* validity*/ 1);

        book_transfer(&mut svm, 
            /* inst data */ &book_transfer_inst_data, 
            /* authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);

        // Assert vault and deposit
        let vault_data = find_vault_data_pda(&alice.pubkey(), create_vault_inst_data.index(), &mint.pubkey(), &TOKEN_PROGRAM);
        let vault = find_vault_pda(&vault_data.0);
        let transfer = find_transfer_pda(&vault_data.0, book_transfer_inst_data.transfer_index());
        let deposit = find_deposit_pda(&transfer.0);

    }
    // Helpers 

    fn create_svm() -> LiteSVM {
        let pime_program_path = Path::join(&current_dir().unwrap(), "target/sbpf-solana-solana/release/pime.so");
        let mut svm = LiteSVM::new();
        if let Err(e) = svm.add_program_from_file(PIME_ID, pime_program_path) {
            panic!("Could not add Pime program: {}", e);
        }
        svm
    }
    
    fn initialize_mint(svm: &mut LiteSVM, authority: &Pubkey, mint: &Keypair, payer: &Keypair) {

        let create_mint_account_inst = solana_system_interface::instruction::create_account(
            /* from */ &payer.pubkey(), 
            /* to */ &mint.pubkey(),
            /* lamports */ svm.minimum_balance_for_rent_exemption(Mint::LEN),
            /* space */ Mint::LEN as u64,
            /* owner */ &TOKEN_PROGRAM,
        );

        let init_mint_inst = spl_token_interface::instruction::initialize_mint(
            /* token program */ &TOKEN_PROGRAM, 
            /* mint */ &mint.pubkey(), 
            /* mint authority */ authority, 
            /* freeze authority */ None, 
            /* decimals */ 6,
        ).unwrap();

        let tx = Transaction::new(
            /* Signers */ &[&payer, &mint], 
            Message::new(
                &[
                    create_mint_account_inst,
                    init_mint_inst
                ],
                /* Payer */ Some(&payer.pubkey())
            ), 
            svm.latest_blockhash()
        );

        if let Err(e) = svm.send_transaction(tx) {
            panic!("Failed to create mint: {:#?}", e);
        }
    }

    fn mint_to(svm: &mut LiteSVM, mint: &Pubkey, mint_authority: &Keypair, to: &Pubkey, to_ata: &Pubkey, payer: &Keypair, amount: u64) {

        let create_impo_inst = create_associated_token_account_idempotent(&payer.pubkey(), to, mint, &TOKEN_PROGRAM);

        let mint_to_inst = spl_token_interface::instruction::mint_to(
            /* token program */ &TOKEN_PROGRAM,  
            /* mint */ mint, 
            /* account */ to_ata, 
            /* owner */ &mint_authority.pubkey(), 
            /* signer pubkeys */ &[&mint_authority.pubkey()], 
            /* amount */ amount 
        ).unwrap();

        let tx = Transaction::new(
            /* Signers */ &[mint_authority, payer], 
            Message::new(
                &[
                    create_impo_inst,
                    mint_to_inst,
                ],
                /* Payer */ Some(&payer.pubkey())
            ), 
            svm.latest_blockhash()
        );
        if let Err(e) = svm.send_transaction(tx) {
            panic!("Failed to mint: {:#?}", e);
        }
        let to_ata_account = svm.get_account(to_ata).unwrap();
        let to_ata_token_account = TokenAccount::unpack(&to_ata_account.data).unwrap();
        assert_eq!(to_ata_token_account.amount, amount);
    }

    fn create_new_vault(
        svm: &mut LiteSVM, 
        authority: &Keypair, 
        instuction_data: &CreateVaultInstructionData,
        mint: &Pubkey, 
    ) {
        let vault_data = find_vault_data_pda(
            &authority.pubkey(), 
            instuction_data.index(), 
            mint, 
            &TOKEN_PROGRAM);
        let vault = find_vault_pda(&vault_data.0);

        let max_transactions = instuction_data.max_transactions();

        let mut data = [0; size_of::<CreateVaultInstructionData>()];
        shared::serialize(instuction_data, &mut data).unwrap();

        let create_vault_inst = Instruction::new_with_bytes(
            /* program id*/     PIME_ID, 
            /* data */          data.as_slice(), 
            /* accounts */ [
                AccountMeta::new(authority.pubkey(), true),
                AccountMeta::new(vault_data.0, false),
                AccountMeta::new(vault.0, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM, false),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            ].to_vec()
        );

        if let Err(e) = svm.send_transaction(Transaction::new(
            /* from keypairs */     &[&authority], 
            /* message */           Message::new(
                /* instructions */      &[create_vault_inst],
                /* payer */             Some(&authority.pubkey())),
            /* latest blockhash */  svm.latest_blockhash()
        )) {
            panic!("{:#?}", e);
        };

        let vault_data_bytes = svm.get_account(&vault_data.0).unwrap().data;
        assert_eq!(vault_data_bytes.len(), size_of::<VaultData>() + size_of::<VaultHistory>() * max_transactions as usize);
        // # SAFETY Data bytes are of type Vault
        let vault_acc = unsafe { pime::states::from_bytes::<VaultData>(&vault_data_bytes[0.. size_of::<VaultData>()]) };
        assert_eq!(vault_acc.unwrap().timeframe(), instuction_data.timeframe());
        assert_eq!(vault_acc.unwrap().max_transactions(), instuction_data.max_transactions());
        assert_eq!(vault_acc.unwrap().max_amount(), instuction_data.max_amount());
        for d in &vault_data_bytes[size_of::<VaultData>() ..] {
            assert_eq!(*d, 0); // Check that transaction history bytes are 0'd out.
        }
    }
    
    fn deposit_to_vault(svm: &mut LiteSVM, from_acc: &Pubkey, from_authority: &Keypair, mint: &Pubkey, inst: &DepositToVaultInstructionData) {
        let mut buf = [0; size_of::<DepositToVaultInstructionData>()];
        shared::serialize(inst, &mut buf).unwrap();
        println!("deposit instruction inst index: {}, amount: {}", inst.index(), inst.amount());
        println!("deposit instruction bytes {:?}", buf);

        let vault_owner = Pubkey::new_from_array(inst.vault_owner());
        let vault_data = find_vault_data_pda(&vault_owner, inst.index(), mint, &TOKEN_PROGRAM).0;
        let vault = find_vault_pda(&vault_data).0;

        let deposit_inst = Instruction::new_with_bytes(
            PIME_ID,
            buf.as_slice(), 
            [
                AccountMeta::new(from_authority.pubkey(), true),
                AccountMeta::new(*from_acc, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM, false),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            ].to_vec());

        let tx = Transaction::new(
            &[&from_authority], 
            Message::new(&[deposit_inst], Some(&from_authority.pubkey())), 
            svm.latest_blockhash());
        if let Err(e) = svm.send_transaction(tx) {
            panic!(" Failed to deposit to vault using token interface: {:#?}", e);
        }

        let vault_acc = svm.get_account(&vault).unwrap();
        let vault_token = TokenAccount::unpack(&vault_acc.data).unwrap();
        assert_eq!(vault_token.amount, inst.amount());
    }

    fn withdraw_from_vault(
        svm: &mut LiteSVM, 
        vault_authority: &Keypair, 
        to: &Pubkey,
        mint: &Pubkey, 
        token_program: &Pubkey, 
        inst: &WithdrawFromVaultInstructionData) {

        let inst_bytes = as_bytes(inst);

        let vault_data = find_vault_data_pda(
            &vault_authority.pubkey(), 
            inst.vault_index(), 
            mint, 
            token_program).0;
        let vault = find_vault_pda(&vault_data).0;

        let to_ata_account = TokenAccount::unpack(&svm.get_account(to).unwrap().data).unwrap();
        let to_pre_amount = to_ata_account.amount;
        let vault_account = TokenAccount::unpack(&svm.get_account(&vault).unwrap().data).unwrap();
        let vault_pre_amount = vault_account.amount;


        let withdraw_inst = Instruction::new_with_bytes(
            PIME_ID, 
            inst_bytes, 
            [
                AccountMeta::new(vault_authority.pubkey(), true),
                AccountMeta::new(vault_data, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(*to, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(*token_program, false),
            ].to_vec());

        let tx = Transaction::new(
            &[vault_authority], 
            Message::new(
                &[withdraw_inst], 
                Some(&vault_authority.pubkey())
            ), 
            svm.latest_blockhash());

        if let Err(e) = svm.send_transaction(tx) {
            panic!("Failed to withdraw: {:#?}", e);
        }

        let to_ata_account = TokenAccount::unpack(&svm.get_account(to).unwrap().data).unwrap();
        let vault_account = TokenAccount::unpack(&svm.get_account(&vault).unwrap().data).unwrap();

        assert_eq!(to_pre_amount + inst.amount(), to_ata_account.amount);
        assert_eq!(vault_account.amount, vault_pre_amount - inst.amount());
    }

    fn book_transfer(svm: &mut LiteSVM, inst_data: &BookTransferInstructionData, authority: &Keypair, mint: &Pubkey, token_program: &Pubkey ) {
        let inst_bytes = as_bytes(inst_data);
        let vault_data = find_vault_data_pda(&authority.pubkey(), inst_data.vault_index(), mint, token_program);
        let vault = find_vault_pda(&vault_data.0);
        let transfer = find_transfer_pda(&vault_data.0, inst_data.transfer_index());
        let deposit = find_deposit_pda(&transfer.0);

        let vault_acc_pre_val = if let Some(a) = &svm.get_account(&vault.0) {
            TokenAccount::unpack(&a.data).unwrap().amount
        }
        else {
            0
        };
        let deposit_acc_pre_val = if let Some(a) = &svm.get_account(&deposit.0) {
            TokenAccount::unpack(&a.data).unwrap().amount
        }
        else {
            0
        };

        let inst = Instruction::new_with_bytes(
            PIME_ID, 
            inst_bytes, 
            [
                AccountMeta::new(authority.pubkey(), true),
                AccountMeta::new(vault_data.0, false),
                AccountMeta::new(vault.0, false),
                AccountMeta::new(transfer.0, false),
                AccountMeta::new(deposit.0, false),
                AccountMeta::new_readonly(*mint, false),
                AccountMeta::new_readonly(*token_program, false),
                AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            ].to_vec()
        );

        let tx = Transaction::new(
            /* from keypairs */ &[authority], 
            /* message */ Message::new(&[inst], Some(&authority.pubkey())), 
            /* recent_blockhash */ svm.latest_blockhash());

        if let Err(e) = svm.send_transaction(tx) {
            panic!("Failed to book transfer: {:#?}", e);
        }

        let vault_acc = TokenAccount::unpack(&svm.get_account(&vault.0).unwrap().data).unwrap();
        let deposit_acc = TokenAccount::unpack(&svm.get_account(&deposit.0).unwrap().data).unwrap();

        // Check that the tokens are transferred out of the vault.
        assert_eq!(vault_acc.amount, vault_acc_pre_val - inst_data.amount()); 

        // Check that the tokens has been sent to the deposit account.
        assert_eq!(deposit_acc.amount, inst_data.amount());
    }

    fn find_vault_data_pda(authority: &Pubkey, index: u64, mint: &Pubkey, token_program: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[
            VaultData::VAULT_DATA_SEED,
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

    fn find_transfer_pda(vault_data: &Pubkey, index: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[
            &vault_data.to_bytes(),
            &index.to_le_bytes(),
        ],
            &Pubkey::new_from_array(pime::ID))
    }

    fn find_deposit_pda(transfer: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[
            &transfer.to_bytes(),
        ],
            &Pubkey::new_from_array(pime::ID))
    }
}
