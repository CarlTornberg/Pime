#![no_std]

use pinocchio::{
  ProgramResult, account_info::AccountInfo, default_allocator, hint::unlikely, msg, nostd_panic_handler, program_entrypoint, program_error::ProgramError, pubkey::{Pubkey, pubkey_eq}
};
use pinocchio_pubkey::declare_id;

use crate::interface::pime_instruction::PimeInstruction;

pub mod interface;
mod processors;
pub mod shared;
pub mod states;
pub mod errors;

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();
declare_id!("FXvAaHn9TQfDrWZV5X47zYB1r8vcwMPpnDCuTeSafEXw");

pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
    if unlikely(!pubkey_eq(program_id, &ID)) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let [inst, data @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };

    match *inst {
        0 => {
            msg!("Create vault");
            processors::create_vault::process_create_vault(accounts, data)?
        },
        1 => {
            msg!("Deposit");
            processors::deposit_to_vault::process_deposit_to_vault(accounts, data)?
        },
        _ => {return Err(ProgramError::InvalidInstructionData);}
        
    }

  Ok(())
}

