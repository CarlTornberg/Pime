mod vault;
use pinocchio::program_error::ProgramError;
pub use vault::*;

/// Trait can be converted from instruction data byte array.
///
/// # SAFETY
/// Struct must be without padding.
///
/// Tip: Only use u8 and [u8]
pub unsafe trait Transmutable { }

/// Convert a T to a byte array
pub fn as_bytes<T: Transmutable>(data: &T) -> &[u8] {
    // SAFETY: Must be Transmutable
    unsafe {
        core::slice::from_raw_parts(
            (data as *const T) as *const u8,
            core::mem::size_of::<T>())
    }
}

/// Convert a data array to T
///
/// # SAFETY
/// Caller must ensure that the provided data is a valid representation of T 
pub unsafe fn from_bytes<T: Transmutable>(data: &[u8]) -> Result<&T, ProgramError> {
    if data.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    // Another way of casting, however, to owned variable.
    // Seems to be doing the same "ptr as *const T" casting behind the scenes.
    // Don't know why use one over another other than one during Ralf Jungs talk at Zurisee Meetup
    // c claims that "it does too much behind the scenes I don't like 'as'"
    // Ok(core::ptr::from_ref(data).cast::<T>().read())

    // SAFETY: The provided data's length matches T
    Ok( unsafe { &*(data.as_ptr() as *const T) } )
}
