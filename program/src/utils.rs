//! Utilities for nobility program

use crate::error::TitleError;
use borsh::BorshDeserialize;
use solana_program::{borsh::try_from_slice_unchecked, program_error::ProgramError};

/// Deserialize and ignore if the type doesn't read all the bytes in the data
pub fn try_from_slice_checked<T: BorshDeserialize>(
    data: &[u8],
    data_size: usize,
) -> Result<T, ProgramError> {
    if data.len() != data_size {
        return Err(TitleError::DataTypeMismatch.into());
    }

    let result: T = try_from_slice_unchecked(data)?;

    Ok(result)
}
