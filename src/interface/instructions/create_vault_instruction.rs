use crate::interface::pime_instruction::PimeInstruction;

impl PimeInstruction {
    pub fn serialize_create_vault_inst_data(index: u64, timeframe: u64, max_withdraws: u64, max_lamports: u64) -> [u8; 33] {
        // instruction data 
        // - [0]: instruction discriminator
        // - [1..9]: index
        // - [9..17]: timeframe
        // - [17..25]: max withdraws
        // - [25..33]: max lamports
        let mut data = [0; 33];
        // Create vault instruction discriminator 0
        data[1..9].copy_from_slice(&index.to_le_bytes());
        data[9..17].copy_from_slice(&timeframe.to_le_bytes());
        data[17..25].copy_from_slice(&max_withdraws.to_le_bytes());
        data[25..33].copy_from_slice(&max_lamports.to_le_bytes());

        data
    }
}

