#![no_std]
use pinocchio::{
    AccountView, 
    Address, 
    ProgramResult, 
    address::{address_eq, declare_id}, 
    default_allocator, 
    error::ProgramError, 
    hint::unlikely, 
    nostd_panic_handler, 
    program_entrypoint 
};
use solana_program_log::log;

pub mod interface;
mod processors;
pub mod states;
pub mod errors;

program_entrypoint!(process_instruction);
default_allocator!();
nostd_panic_handler!();
declare_id!("FXvAaHn9TQfDrWZV5X47zYB1r8vcwMPpnDCuTeSafEXw");

pub fn process_instruction(
  program_id: &Address,
  accounts: &[AccountView],
  instruction_data: &[u8],
) -> ProgramResult {
    if unlikely(!address_eq(program_id, &crate::ID)) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let [inst, data @ ..] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };

    match *inst {
        0 => {
            log!("Create vault");
            processors::create_vault::process_create_vault(accounts, data)?
        },
        1 => {
            log!("Deposit");
            processors::deposit_to_vault::process_deposit_to_vault(accounts, data)?
        },
        2 => {
            log!("Withdraw");
            processors::withdraw_from_vault::process_withdraw_from_vault(accounts, data)?
        },
        3 => {
            log!("Close");
            processors::close_vault::process_close_vault(accounts, data)?
        },
        10 => {
            log!("Book transfer");
            processors::transfer::book_transfer::process_book_transfer(accounts, data)?
        },
        11 => {
            log!("Execute transfer");
            processors::transfer::execute_transfer::execute_transfer(accounts, data)?
        },
        12 => {
            log!("Unbook transfer");
            processors::transfer::unbook_transfer::unbook_transfer(accounts, data)?
        },
        _ => {return Err(ProgramError::InvalidInstructionData);}

    }

  Ok(())
}

