pub mod create_vault_instruction;


use pinocchio::program_error::ProgramError;


#[repr(u8)]
pub enum PimeInstruction {

    /// Creates a new vault.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The authority of the new vault
    ///   1. `[writeable]` The vault data account.
    ///   2. `[writeable]` The vault account.
    ///   3. `[]` The mint address of the vault. 
    ///   4. `[]` The token program. 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The vaults unique index. Allows for multiple vaults of one mint.
    ///   - `u64` The timeframe, in ms, which the vault's restriction encompasses.
    ///   - `u64` The number of withdraws allowed within a timeframe.
    ///   - `u64` The number of lamports allowed to be withdrawn within a timeframe.
    CreateVault = 0,
    DepositToVault = 1,
    WithdrawFromVault = 2,
    CloseVault = 3,
    BookTransfer = 4,
    ExecuteTransfer = 5,
    UnbookTransfer = 6,
}

impl TryFrom<u8> for PimeInstruction {
    type Error = ProgramError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value { 
            0..=6 => Ok(unsafe { core::mem::transmute::<u8, PimeInstruction>(value) }),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}

// TODO! Perform exhaustive convertion tests.
