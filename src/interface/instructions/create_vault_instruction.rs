#[cfg(test)]
pub mod helpers {
    use solana_sdk::{message::{AccountMeta, Instruction}, pubkey::Pubkey};

    use crate::states::Vault;

    pub fn create_vault_instruction(
        authority: Pubkey,
        mint: Pubkey,
        token_program: Pubkey,
        index: u64,
        timeframe: u64,
        max_withdraws: u64,
        max_lamport_withdraw: u64,
    ) -> Instruction {
        let mut accounts = [AccountMeta::new(authority, true)].to_vec();

        accounts.push(AccountMeta::new_readonly(mint, false));
        accounts.push(AccountMeta::new_readonly(token_program, false));

        Instruction { 
            program_id: Pubkey::new_from_array(crate::ID), 
            accounts: [
                AccountMeta::new(authority, true),
                AccountMeta::new(mint, false),
                AccountMeta::new(token_program, false),
            ].to_vec(), 
            data: [].to_vec(), 
        }
    }
}
