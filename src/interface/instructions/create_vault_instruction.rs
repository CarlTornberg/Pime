#[cfg(test)]
pub mod helpers {
    use pinocchio_system::instructions::CreateAccount;
    use solana_sdk::{message::{AccountMeta, Instruction}, pubkey::Pubkey};

    use crate::{interface::instructions::PimeInstruction, states::Vault};



    pub fn create_vault_instruction(
        authority: Pubkey,
        mint: Pubkey,
        token_program: Pubkey,
        index: u64,
        timeframe: u64,
        max_withdraws: u64,
        max_lamport_withdraw: u64,
    ) -> Instruction {
        let vault_data = Vault::get_vault_data_pda(authority.as_array(), index, mint.as_array(), token_program.as_array());;
        let vault = Vault::get_vault_pda(&vault_data.0, mint.as_array(), token_program.as_array());

        // instruction data 
        // - [0]: instruction discriminator
        // - [1..5]: index
        // - [5..9]: timeframe
        // - [9..13]: max withdraws
        // - [13..17]: max lamport withdraw
        let mut data = [0; 17];
        // Create Vault instruction i 0
        data[1..5].copy_from_slice(&index.to_le_bytes());
        // Skip the rest for now since they are not strictly needed in the function call.

        Instruction { 
            program_id: Pubkey::new_from_array(crate::ID), 
            accounts: [
                AccountMeta::new(authority, true),
                AccountMeta::new(vault_data.0.into(), false),
                AccountMeta::new(vault.0.into(), false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(token_program, false),
            ].to_vec(),
            data: data.to_vec(),
        }
    }
}
