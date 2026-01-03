use pinocchio::program_error::ProgramError;


#[repr(u8)]
pub enum PimeInstruction {

    /// Creates a new vault.
    ///
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]`     The authority of the new vault
    ///   1. `[writeable]`  The vault data account.
    ///   2. `[writeable]`  The vault account.
    ///   3. `[]`           The mint address of the vault. 
    ///   4. `[]`           The token program. 
    ///   5. `[]`           The system program. 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64` The vaults unique index. Allows for multiple vaults of one mint.
    ///   - `u64` The timeframe, in ms, which the vault's restriction encompasses.
    ///   - `u64` The number of withdraws allowed within a timeframe.
    ///   - `u64` The number of lamports allowed to be withdrawn within a timeframe.
    CreateVault = 0,

    /// Deposit tokens to a vault
    ///
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]`     The signer, and authority of the token account. 
    ///   1. `[writeable]`  The token account which the tokens will be withdrawn from
    ///   1. `[writeable]`  The vault account.
    ///   2. `[]`           The mint address of the vault. 
    ///   3. `[]`           The token program. 
    ///   4. `[]`           The system program. (Optional, if vault needs to be initialized) 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `[u8; 32]`  The vault owners' public key.
    ///   - `u64`       The vault's vault's index.
    ///   - `u64`       The amount to transfer in lamports (without decimals).
    DepositToVault = 1,

    /// Withdraw tokens to a vault
    ///
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]`     The owner of the vault.
    ///   1. `[writeable]`  The vault data account.
    ///   1. `[writeable]`  The vault account.
    ///   2. `[]`           The mint address of the vault. 
    ///   3. `[]`           The token program. 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64`       The amount to withdraw in lamports (without decimals).
    WithdrawFromVault = 2,

    // CloseVault = 3,

    /// Book a transfer.
    ///
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]`     The owner of the vault.
    ///   1. `[]`  The vault data account.
    ///   1. `[writeable]`  The vault account.
    ///   2. `[writeable]`  The transfer account.
    ///   3. `[writeable]`  The deposit account.
    ///   4. `[]`           The mint address of the vault/transfer. 
    ///   5. `[]`           The token program. 
    ///   6. `[]`           The system program. 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64`       The amount to transfer (without decimals).
    ///   - `Pubkey`    Destination account.
    ///   - `u64`       The vault index.
    ///   - `u64`       The transfer index.
    ///   - `UnixTimestamp` Warmup period
    ///   - `UnixTimestamp` Validity period
    BookTransfer = 10,

    /// Execute a transfer.
    ///
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]`     The owner of the vault.
    ///   1. `[writeable]`  The vault account.
    ///   2. `[writeable]`  The transfer account.
    ///   3. `[writeable]`  The deposit account.
    ///   4. `[]`           The mint address of the vault/transfer. 
    ///   5. `[]`           The token program. 
    ///
    /// Data expected by this instruction:
    ///
    ///   - `u64`       The vault index.
    ///   - `u64`       The transfer index.
    ExecuteTransfer = 11,

    UnbookTransfer = 12,
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

// TODO! Perform exhaustive conversion tests.
