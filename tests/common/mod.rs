// Helpers 

    use std::env::current_dir;
    use std::path::Path;

    use litesvm::LiteSVM;
    use litesvm::types::TransactionResult;
    use pime::interface::instructions::book_transfer::BookTransferInstructionData;
    use pime::interface::instructions::close_vault_instruction::CloseVaultInstructionData;
    use pime::interface::instructions::create_vault_instruction::CreateVaultInstructionData;
    use pime::interface::instructions::deposit_to_vault_instruction::DepositToVaultInstructionData;
    use pime::interface::instructions::execute_transfer::ExecuteTransferInstructionData;
    use pime::interface::instructions::unbook_transfer_instruction::UnbookTransferInstructionData;
    use pime::interface::instructions::withdraw_from_vault::WithdrawFromVaultInstructionData;
    use pime::states::transfer_data::TransferData;
    use pime::states::{Transmutable, VaultData, VaultHistory, as_bytes, from_bytes};
    use pinocchio::sysvars::clock::UnixTimestamp;
    use solana_sdk::message::{AccountMeta, Instruction};
    use solana_sdk::pubkey::PUBKEY_BYTES;
    use solana_sdk::{message::Message, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
    use spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent;
    use spl_token_interface::state::Mint;
    use spl_token_interface::state::Account as TokenAccount;

    const PIME_ID: Pubkey = Pubkey::new_from_array(pime::ID);
    const TOKEN_PROGRAM: Pubkey = spl_token_interface::ID;

pub fn create_svm() -> LiteSVM {
    let pime_program_path = Path::join(&current_dir().unwrap(), "target/sbpf-solana-solana/release/pime.so");
    let mut svm = LiteSVM::new();
    if let Err(e) = svm.add_program_from_file(PIME_ID, pime_program_path) {
        panic!("Could not add Pime program: {}", e);
    }
    svm
}

pub fn initialize_mint(svm: &mut LiteSVM, authority: &Pubkey, payer: &Keypair, mint: &Keypair, token_program: &Pubkey) -> TransactionResult {

    let create_mint_account_inst = solana_system_interface::instruction::create_account(
        /* from */ &payer.pubkey(), 
        /* to */ &mint.pubkey(),
        /* lamports */ svm.minimum_balance_for_rent_exemption(Mint::LEN),
        /* space */ Mint::LEN as u64,
        /* owner */ token_program,
    );

    let init_mint_inst = spl_token_interface::instruction::initialize_mint(
        /* token program */ token_program, 
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

    svm.send_transaction(tx)
}

pub fn mint_to(
    svm: &mut LiteSVM, 
    amount: u64, 
    payer: &Keypair, 
    to: &Pubkey, 
    to_ata: &Pubkey, 
    mint_authority: &Keypair, 
    mint: &Pubkey, 
    token_program: &Pubkey
) {

    let create_impo_inst = create_associated_token_account_idempotent(&payer.pubkey(), to, mint, token_program);

    let mint_to_inst = spl_token_interface::instruction::mint_to(
        /* token program */ token_program,  
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

pub fn create_new_vault(
    svm: &mut LiteSVM, 
    authority: &Keypair, 
    inst_data: &CreateVaultInstructionData,
    mint: &Pubkey, 
) {
    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let vault = find_vault_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let data = as_bytes(inst_data);

    let create_vault_inst = Instruction::new_with_bytes(
        /* program id*/     PIME_ID, 
        /* data */          data, 
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
    assert_eq!(vault_data_bytes.len(), size_of::<VaultData>() + size_of::<VaultHistory>() * inst_data.max_transactions() as usize);
    // # SAFETY Data bytes are of type Vault
    let vault_acc = from_bytes::<VaultData>(&vault_data_bytes[0.. size_of::<VaultData>()]).unwrap();
    assert_eq!(vault_acc.timeframe(), inst_data.timeframe());
    assert_eq!(vault_acc.max_transactions(), inst_data.max_transactions());
    assert_eq!(vault_acc.max_amount(), inst_data.max_amount());
    assert_eq!(vault_acc.open_transfers(), 0);

    for i in 0.. (inst_data.max_transactions() as usize) {
        let range = VaultData::LEN + i * VaultHistory::LEN;
        let history = from_bytes::<VaultHistory>(&vault_data_bytes[range .. range + VaultHistory::LEN]).unwrap();
        assert_eq!(history.timestamp(), UnixTimestamp::MIN);
        assert_eq!(history.amount(), u64::MIN);
    }
}

pub fn deposit_to_vault(svm: &mut LiteSVM, from_acc: &Pubkey, from_authority: &Keypair, mint: &Pubkey, inst_data: &DepositToVaultInstructionData) {
    let buf = as_bytes(inst_data);
    println!("deposit instruction inst index: {}, amount: {}", inst_data.vault_index(), inst_data.amount());
    println!("deposit instruction bytes {:?}", buf);

    let vault = find_vault_pda(
        inst_data.vault_index(), 
        from_authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let deposit_inst = Instruction::new_with_bytes(
        PIME_ID,
        buf, 
        [
            AccountMeta::new(from_authority.pubkey(), true),
            AccountMeta::new(*from_acc, false),
            AccountMeta::new(vault.0, false),
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

    let vault_acc = svm.get_account(&vault.0).unwrap();
    let vault_token = TokenAccount::unpack(&vault_acc.data).unwrap();
    assert_eq!(vault_token.amount, inst_data.amount());
}

pub fn withdraw_from_vault(
    svm: &mut LiteSVM, 
    authority: &Keypair, 
    to: &Pubkey,
    mint: &Pubkey, 
    token_program: &Pubkey, 
    inst_data: &WithdrawFromVaultInstructionData) {

    let inst_bytes = as_bytes(inst_data);

    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let vault = find_vault_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let to_ata_account = TokenAccount::unpack(&svm.get_account(to).unwrap().data).unwrap();
    let to_pre_amount = to_ata_account.amount;
    let vault_account = TokenAccount::unpack(&svm.get_account(&vault.0).unwrap().data).unwrap();
    let vault_pre_amount = vault_account.amount;


    let withdraw_inst = Instruction::new_with_bytes(
        PIME_ID, 
        inst_bytes, 
        [
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_data.0, false),
            AccountMeta::new(vault.0, false),
            AccountMeta::new(*to, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*token_program, false),
        ].to_vec());

    let tx = Transaction::new(
        &[authority], 
        Message::new(
            &[withdraw_inst], 
            Some(&authority.pubkey())
        ), 
        svm.latest_blockhash());

    if let Err(e) = svm.send_transaction(tx) {
        panic!("Failed to withdraw: {:#?}", e);
    }

    let to_ata_account = TokenAccount::unpack(&svm.get_account(to).unwrap().data).unwrap();
    let vault_account = TokenAccount::unpack(&svm.get_account(&vault.0).unwrap().data).unwrap();

    assert_eq!(to_pre_amount + inst_data.amount(), to_ata_account.amount);
    assert_eq!(vault_account.amount, vault_pre_amount - inst_data.amount());
}

pub fn close_vault(svm: &mut LiteSVM, inst_data: &CloseVaultInstructionData, authority: &Keypair, mint: &Pubkey, token_program: &Pubkey) {
    let inst_bytes = as_bytes(inst_data);

    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let vault = find_vault_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let inst = Instruction::new_with_bytes(
        PIME_ID, 
        inst_bytes, 
        [
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault.0, false),
            AccountMeta::new(vault_data.0, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*token_program, false)
        ].to_vec());

    let tx = Transaction::new(
        &[authority], 
        Message::new(
            &[inst], 
            Some(&authority.pubkey())
        ), 
        svm.latest_blockhash()
    );

    if let Err(e) = svm.send_transaction(tx) {
        panic!("Failed to close vault: {:#?}", e);
    }

    assert_eq!(svm.get_account(&vault.0), None);
    assert_eq!(svm.get_account(&vault_data.0), None);
}

pub fn book_transfer(svm: &mut LiteSVM, inst_data: &BookTransferInstructionData, authority: &Keypair, mint: &Pubkey, token_program: &Pubkey ) {
    let inst_bytes = as_bytes(inst_data);

    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let vault = find_vault_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let transfer = find_transfer_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        &inst_data.destination,
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let deposit = find_deposit_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        &inst_data.destination,
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let vault_data_open_transfers_pre = if let Some(a) = &svm.get_account(&vault_data.0) {
        from_bytes::<VaultData>(&a.data[..VaultData::LEN]).unwrap().open_transfers()
    }
    else { 0 };

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
    let t = svm.get_account(&transfer.0).unwrap();
    let transfer_acc = from_bytes::<TransferData>(&t.data).unwrap();

    assert_eq!(transfer_acc.amount(), inst_data.amount());

    // Check that the tokens are transferred out of the vault.
    assert_eq!(vault_acc.amount, vault_acc_pre_val - inst_data.amount()); 

    // Check that the tokens has been sent to the deposit account.
    assert_eq!(deposit_acc.amount, deposit_acc_pre_val + inst_data.amount());

    // Verify that Vault data incremented the newly added transfer
    if let Some(a) = svm.get_account(&vault_data.0) {
        assert_eq!(
            from_bytes::<VaultData>(&a.data[..VaultData::LEN]).unwrap().open_transfers(),
            vault_data_open_transfers_pre + 1);
    }
}

pub fn execute_transfer(svm: &mut LiteSVM, inst_data: &ExecuteTransferInstructionData, authority: &Keypair, destination: &Pubkey, destination_owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) {
    let inst_bytes = as_bytes(inst_data);

    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let transfer = find_transfer_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        destination.as_array(),
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let deposit = find_deposit_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        destination.as_array(),
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let deposit_acc_pre_val = if let Some(a) = &svm.get_account(&deposit.0) {
        TokenAccount::unpack(&a.data).unwrap().amount
    }
    else { 0 };
    let destination_acc_pre_val = if let Some(a) = &svm.get_account(destination) {
        TokenAccount::unpack(&a.data).unwrap().amount
    }
    else { 0 };
    let pre_open_transfers = from_bytes::<VaultData>(&svm.get_account(&vault_data.0).unwrap().data[..VaultData::LEN]).unwrap().open_transfers();
    let tda = svm.get_account(&transfer.0).unwrap();
    let transfer_acc = from_bytes::<TransferData>(&tda.data).unwrap();
    let transfer_amount = transfer_acc.amount();

    let inst = Instruction::new_with_bytes(
        PIME_ID, 
        inst_bytes, 
        [
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault_data.0, false),
            AccountMeta::new(transfer.0, false),
            AccountMeta::new(deposit.0, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(solana_system_interface::program::ID, false),
            AccountMeta::new_readonly(*destination_owner, false),
            AccountMeta::new_readonly(spl_associated_token_account_interface::program::ID, false),
        ].to_vec()
    );

    let tx = Transaction::new(
        /* from keypairs */ &[authority], 
        /* message */ Message::new(&[inst], Some(&authority.pubkey())), 
        /* recent_blockhash */ svm.latest_blockhash());

    if let Err(e) = svm.send_transaction(tx) {
        panic!("Failed to execute transfer: {:#?}", e);
    }

    // Check that the deposit account has transferred the tokens out.
    if let Some(a) = svm.get_account(&deposit.0) { // If account is not closed for some reason.
        assert_eq!(TokenAccount::unpack(&a.data).unwrap().amount, deposit_acc_pre_val - transfer_amount);
    }

    // Check that the destination has received its tokens
    if let Some(a) = svm.get_account(destination) {
        assert_eq!(TokenAccount::unpack(&a.data).unwrap().amount, destination_acc_pre_val + transfer_amount);
    }

    // Check that the transfer account is closed.
    if let Some(a) = svm.get_account(&transfer.0) {
        assert_eq!(a.lamports, 0);
    }

    let post_open_transfers = from_bytes::<VaultData>(&svm.get_account(&vault_data.0).unwrap().data[..VaultData::LEN]).unwrap().open_transfers();
    assert_eq!(post_open_transfers, pre_open_transfers - 1);
}

pub fn unbook_transfer(svm: &mut LiteSVM, inst_data: &UnbookTransferInstructionData, authority: &Keypair, mint: &Pubkey, token_program: &Pubkey) {

    let vault = find_vault_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let vault_data = find_vault_data_pda(
        inst_data.vault_index(), 
        authority.pubkey().as_array(), 
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let transfer = find_transfer_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        &inst_data.destination,
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );
    let deposit = find_deposit_pda(
        inst_data.vault_index(), 
        inst_data.transfer_index(),
        authority.pubkey().as_array(),
        &inst_data.destination,
        mint.as_array(),
        TOKEN_PROGRAM.as_array()
    );

    let inst_bytes = as_bytes(inst_data);

    let inst = Instruction::new_with_bytes(
        PIME_ID, 
        inst_bytes, 
        [
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(vault.0, false),
            AccountMeta::new(vault_data.0, false),
            AccountMeta::new(transfer.0, false),
            AccountMeta::new(deposit.0, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(*token_program, false),
        ].to_vec());

    let tx = Transaction::new(
        &[authority], 
        Message::new(&[inst], Some(&authority.pubkey())), 
        svm.latest_blockhash()
    );

    if let Err(e) = svm.send_transaction(tx) {
        panic!("Failed to unbook transfer: {:#?}", e);
    }

    if svm.get_account(&transfer.0).is_some() {
        panic!("Failed to close transfer account");
    }

    if svm.get_account(&deposit.0).is_some() {
        panic!("Failed to close deposit account");
    }
}

pub fn find_vault_data_pda(vault_index: u64, authority: &[u8; PUBKEY_BYTES], mint: &[u8; PUBKEY_BYTES], token_program: &[u8; PUBKEY_BYTES]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        VaultData::VAULT_DATA_SEED,
        &vault_index.to_le_bytes(),
        authority,
        mint,
        token_program,
    ], 
        &Pubkey::new_from_array(pime::ID))
}

pub fn find_vault_pda(vault_index: u64, authority: &[u8; PUBKEY_BYTES], mint: &[u8; PUBKEY_BYTES], token_program: &[u8; PUBKEY_BYTES]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        VaultData::VAULT_SEED,
        &vault_index.to_le_bytes(),
        authority,
        mint,
        token_program,
    ],
        &Pubkey::new_from_array(pime::ID))
}

pub fn find_transfer_pda(vault_index: u64, transfer_index: u64, authority: &[u8; PUBKEY_BYTES], destination: &[u8; PUBKEY_BYTES], mint: &[u8; PUBKEY_BYTES], token_program: &[u8; PUBKEY_BYTES] ) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        TransferData::TRANSFER_SEED,
        &vault_index.to_le_bytes(),
        &transfer_index.to_le_bytes(),
        authority,
        destination,
        mint,
        token_program,
    ],
        &Pubkey::new_from_array(pime::ID))
}

pub fn find_deposit_pda(vault_index: u64, transfer_index: u64, authority: &[u8; PUBKEY_BYTES], destination: &[u8; PUBKEY_BYTES], mint: &[u8; PUBKEY_BYTES], token_program: &[u8; PUBKEY_BYTES] ) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[
        TransferData::DEPOSIT_SEED,
        &vault_index.to_le_bytes(),
        &transfer_index.to_le_bytes(),
        authority,
        destination,
        mint,
        token_program,
    ],
        &Pubkey::new_from_array(pime::ID))
}
