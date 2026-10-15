//! What can go wrong while a form is filled or read.

use std::io;

/// An error of filling a form or reading its values.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The terminal could not be read or written.
    #[error("terminal: {0}")]
    Io(#[from] io::Error),
    /// The input ended, as a closed standard input does, before the field had a value.
    #[error("the input ended before field `{0}` was filled")]
    EndOfInput(String),
    /// The user cancelled a choice or a password with Esc or Ctrl+C.
    #[error("field `{0}` was cancelled")]
    Cancelled(String),
    /// A select or multiselect field was given no options.
    #[error("field `{0}` has no options to choose from")]
    NoOptions(String),
    /// A preset answer, given where nobody can be asked again, was refused.
    #[error("field `{name}`: {message}")]
    Refused {
        /// The name of the field.
        name: String,
        /// Why the answer was refused.
        message: String,
    },
    /// The form has no field of that name.
    #[error("no field named `{0}`")]
    NotFound(String),
    /// The field has not been filled yet.
    #[error("field `{0}` is not filled yet")]
    NotFilled(String),
    /// The field was not asked, since its condition did not hold.
    #[error("field `{0}` was skipped")]
    Skipped(String),
    /// The field holds a value of another type than the one asked for.
    #[error("field `{name}` does not hold a `{expected}`")]
    WrongType {
        /// The name of the field.
        name: String,
        /// The type that was asked for.
        expected: &'static str,
    },
}
