use crate::interface::pime_instruction::PimeInstruction;

impl PimeInstruction {
    pub fn serialize_create_vault_inst_data(index: u64, timeframe: u64, max_withdraws: u64, max_lamport_withdraw: u64) -> [u8; 33] {
        // instruction data 
        // - [0]: instruction discriminator
        // - [1..9]: index
        // - [9..17]: timeframe
        // - [17..25]: max withdraws
        // - [25..33]: max lamport withdraw
        let mut data = [0; 33];
        // Create Vault instruction i 0
        data[1..9].copy_from_slice(&index.to_le_bytes());
        // Skip the rest for now since they are not strictly needed in the function call.

        data
    }
    
}
