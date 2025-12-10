use crate::interface::pime_instruction::PimeInstruction;

impl PimeInstruction {
    pub fn serialize_deposit_to_vault_instruction(index: u64, lamports: u64) -> [u8; 17] {
        // instruction data 
        // - [0]: instruction discriminator
        // - [1..9]: index
        // - [9..17]: lamports
        let mut data = [1; 17];
        // Deposit to vault instruction discriminator: 1
        data[1..9].copy_from_slice(&index.to_le_bytes());
        data[9..17].copy_from_slice(&lamports.to_le_bytes());

        data
    }
}

