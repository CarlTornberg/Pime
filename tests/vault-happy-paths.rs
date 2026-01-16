mod common;
#[cfg(test)]
mod happy_paths_tests {

    use super::common::*;

    use pime::interface::instructions::book_transfer::BookTransferInstructionData;
    use pime::interface::instructions::close_vault_instruction::CloseVaultInstructionData;
    use pime::interface::instructions::create_vault_instruction::CreateVaultInstructionData;
    use pime::interface::instructions::deposit_to_vault_instruction::DepositToVaultInstructionData;
    use pime::interface::instructions::execute_transfer::ExecuteTransferInstructionData;
    use pime::interface::instructions::unbook_transfer_instruction::UnbookTransferInstructionData;
    use pime::interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData;
    use solana_sdk::{native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer};
    use spl_associated_token_account_interface::address::{get_associated_token_address, get_associated_token_address_with_program_id};
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
            /* allows transfers */ true,
            /* transfer min window */ 4u64,
            /* transfer max_window */ 5u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);

        // Create vault based on the new mint
        create_new_vault(&mut svm, &alice, &create_vault_instruction_data, &mint.pubkey());

    }

    #[test]
    fn alice_closes_own_vault() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();

        let create_vault_instruction_data = CreateVaultInstructionData::new(
            /* index */ 0u64, 
            /* timeframe */ 1i64, 
            /* max_withdraws */ 2u64, 
            /* max_lamports */ 3u64,
            /* allows transfers */ true,
            /* transfer min window */ 4u64,
            /* transfer max_window */ 5u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);

        // Create vault based on the new mint
        create_new_vault(&mut svm, &alice, &create_vault_instruction_data, &mint.pubkey());

        let close_inst = CloseVaultInstructionData::new(create_vault_instruction_data.vault_index());

        close_vault(&mut svm, &close_inst, &alice, &mint.pubkey(), &TOKEN_PROGRAM);
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
            /* allows transfers */ true,
            /* transfer min window */ 4u64,
            /* transfer max_window */ 5u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);

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
            /* amount */ mint_amount, 
            /* payer */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* mint_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM
        );

        let deposit_amount = 4_000;
        let deposit_inst = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* vault index */ create_vault_instruction_data.vault_index(), 
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
            /* timeframe */ 0i64, 
            /* max_withdraws */ 10u64, 
            /* max_lamports */ 10u64,
            /* allows transfers */ true,
            /* transfer min window */ 5u64,
            /* transfer max_window */ 6u64,
        );

        // Create new mint
        let mint = Keypair::new();
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);

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
            /* amount */ mint_amount, 
            /* payer */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* mint_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM
        );

        let deposit_amount = 4_000;
        let deposit_inst = DepositToVaultInstructionData::new(
            /* vault owner */ alice.pubkey().to_bytes(), 
            /* vault index */ create_vault_instruction_data.vault_index(), 
            /* deposit amount */ deposit_amount);

        deposit_to_vault(
            &mut svm, 
            /* from */ &alice_ata, 
            /* from_authority */ &alice, 
            &mint.pubkey(), 
            &deposit_inst);

        let withdraw_inst = WithdrawFromVaultInstructionData::new(
            /* amount */ create_vault_instruction_data.max_amount(), 
            /* vault index */ create_vault_instruction_data.vault_index());

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
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);
        mint_to(
            /* svm */ &mut svm, 
            /* amount */ mint_amount, 
            /* payer */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* mint_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM
        );

        // Create vault
        let create_vault_inst_data = CreateVaultInstructionData::new(
            /* index */ 1, 
            /* timeframe */ 2, 
            /* max_transactions */ 3, 
            /* max_amount */ 4, 
            /* allows transfers */ true,
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
            /* vault_index */ create_vault_inst_data.vault_index(), 
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
            /* vault_index */ create_vault_inst_data.vault_index(), 
            /* transfer_index */ 1, 
            /* warmup */ 1, 
            /* validity*/ 1);

        book_transfer(&mut svm, 
            /* inst data */ &book_transfer_inst_data, 
            /* authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);

        // Assert vault and deposit
        let vault = find_vault_pda(
            book_transfer_inst_data.vault_index(), 
            alice.pubkey().as_array(), 
            mint.pubkey().as_array(),
            TOKEN_PROGRAM.as_array()
        );
        let deposit = find_deposit_pda(
            book_transfer_inst_data.vault_index(), 
            book_transfer_inst_data.transfer_index(),
            alice.pubkey().as_array(),
            &book_transfer_inst_data.destination,
            mint.pubkey().as_array(),
            TOKEN_PROGRAM.as_array()
        );
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

        let destination = Keypair::new();
        let destination_ata = get_associated_token_address(&destination.pubkey(), &mint.pubkey());

        let mint_amount = 1_000;

        // Mint tokens
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);
        mint_to(
            /* svm */ &mut svm, 
            /* amount */ mint_amount, 
            /* payer */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* mint_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM
        );
        
        // Create vault
        let create_vault_inst_data = CreateVaultInstructionData::new(
            /* index */ 1, 
            /* timeframe */ 2, 
            /* max_transactions */ 3, 
            /* max_amount */ 4, 
            /* allows transfers */ true,
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
            /* vault index */ create_vault_inst_data.vault_index(), 
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
            /* destination */ destination_ata.to_bytes(),
            /* vault_index */ create_vault_inst_data.vault_index(), 
            /* transfer_index */ 1, 
            /* warmup */ 0, 
            /* validity*/ 100);

        book_transfer(&mut svm, 
            /* inst data */ &book_transfer_inst_data, 
            /* authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);

        let execute_transfer_inst_data = ExecuteTransferInstructionData::new(
            book_transfer_inst_data.vault_index(), 
            book_transfer_inst_data.transfer_index());

        execute_transfer(&mut svm, 
            /* inst_data */ &execute_transfer_inst_data, 
            /* authority */ &alice, 
            /* destination */ &destination_ata, 
            /* destination owner */ &destination.pubkey(),
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);
    }

    #[test]
    fn alice_unbooks_transfer() {
        let mut svm = create_svm();
        let alice = Keypair::new();
        svm.airdrop(&alice.pubkey(), LAMPORTS_PER_SOL).unwrap();
        let mint = Keypair::new();
        let alice_ata = get_associated_token_address(&alice.pubkey(), &mint.pubkey());

        let destination = Keypair::new();
        let destination_ata = get_associated_token_address(&destination.pubkey(), &mint.pubkey());

        let mint_amount = 1_000;

        // Mint tokens
        initialize_mint(&mut svm, &alice.pubkey(), &alice, &mint, &TOKEN_PROGRAM);
        mint_to(
            /* svm */ &mut svm, 
            /* amount */ mint_amount, 
            /* payer */ &alice, 
            /* to */ &alice.pubkey(), 
            /* to_ata */ &alice_ata, 
            /* mint_authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM
        );

        // Create vault
        let create_vault_inst_data = CreateVaultInstructionData::new(
            /* index */ 1, 
            /* timeframe */ 2, 
            /* max_transactions */ 3, 
            /* max_amount */ 4, 
            /* allows transfers */ true,
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
            /* vault index */ create_vault_inst_data.vault_index(), 
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
            /* destination */ destination_ata.to_bytes(),
            /* vault_index */ create_vault_inst_data.vault_index(), 
            /* transfer_index */ 1, 
            /* warmup */ 0, 
            /* validity*/ 100);

        book_transfer(&mut svm, 
            /* inst data */ &book_transfer_inst_data, 
            /* authority */ &alice, 
            /* mint */ &mint.pubkey(), 
            /* token_program */ &TOKEN_PROGRAM);

        let unbook_transfer_inst_data = UnbookTransferInstructionData::new(
            book_transfer_inst_data.vault_index(), 
            book_transfer_inst_data.transfer_index(),
            destination_ata.to_bytes(),
        );
        unbook_transfer(&mut svm, 
            /* inst data */ &unbook_transfer_inst_data, 
            /* authority */ &alice, 
            &mint.pubkey(), 
            &TOKEN_PROGRAM);
    }
}
