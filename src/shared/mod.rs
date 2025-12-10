use pinocchio::{ProgramResult, program_error::ProgramError};
use crate::states::Transmutable;
mod unpack;

/// Packs a vault to its byte format.
pub fn serialize<T: Transmutable + Sized>(data: &T, buf: &mut [u8]) -> ProgramResult{
    if buf.len() != size_of::<T>() {
        return Err(ProgramError::Custom(67));
    }
    buf.copy_from_slice(
        // # SAFETY Vault is Transmutable and of size T
        unsafe {
            core::slice::from_raw_parts(
                data as *const T as *const u8,
                size_of::<T>()
            )
        }
    );
    Ok(())
}

pub fn deserialize<T: Transmutable + Sized>(data: &[u8]) -> Result<&T, ProgramError> {
    if data.len() != size_of::<T>() {
        return Err(ProgramError::Custom(67));
    }
    // # SAFETY : Data is of length T and is Transmutable
    Ok(unsafe { &*(data.as_ptr() as *const T) })
}
