#![no_std]
use pinocchio::{
  ProgramResult, account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey
};
use pinocchio_pubkey::declare_id;

entrypoint!(process_instruction);
declare_id!("11111111111111111111111111111111");

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {

    let [inst, data @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };

    match inst {
        0 => msg!("This is the first instruction"),
        _ => msg!("Hello from my program!"),
    }
  Ok(())
}

