use pinocchio::{ProgramResult, account_info::AccountInfo};

/// Transfers assets from its booked vault to the received.
pub fn process_transfer(accounts: &[AccountInfo], instrution_data: &[u8]) -> ProgramResult {
    
    // Check if transfer is booked and within its timeframe.
    
    ProgramResult::Ok(())
}
