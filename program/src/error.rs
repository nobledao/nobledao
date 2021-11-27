//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TitleError {
    /// Incorrect authority provided on update or delete
    #[error("Incorrect authority provided on update or delete")]
    IncorrectAuthority,

    /// Calculation overflow
    #[error("Calculation overflow")]
    Overflow,

    /// Data type mismatched
    #[error("Data type length mismatched")]
    DataTypeMismatch,
}
impl From<TitleError> for ProgramError {
    fn from(e: TitleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TitleError {
    fn type_of() -> &'static str {
        "Title Error"
    }
}