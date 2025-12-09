use pinocchio::program_error::ProgramError;

pub const fn unpack_u64(instruction_data: &[u8]) -> Result<u64, ProgramError> {
    if instruction_data.len() >= core::mem::size_of::<u64>() {
        Ok(
            u64::from_le_bytes(
                // SAFETY: instruction data min size is at least the size of an u64
                unsafe { *(instruction_data.as_ptr() as *const [u8; core::mem::size_of::<u64>()]) })
        )
    }
    else {
        Err(ProgramError::InvalidInstructionData)
    }
}
