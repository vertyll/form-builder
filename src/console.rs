//! Where a form reads its input: the [`Terminal`], or any other [`Console`], such as one that replays answers in a
//! test.

use crate::Error;
use crate::prompt::{Messages, Prompt};
use crate::selection::Mode;

/// A choice to be made: what [`Console::choose`] shows and starts from.
#[derive(Debug)]
#[non_exhaustive]
pub struct Choice<'a> {
    /// The prompt above the options.
    pub prompt: &'a Prompt,
    /// The labels of the options, in order.
    pub options: &'a [String],
    /// How many options may be chosen.
    pub mode: Mode,
    /// The options the field starts with: the cursor on the first, and all of them chosen in a multiselect.
    pub defaults: &'a [usize],
    /// The texts the form shows of its own.
    pub messages: &'a Messages,
}

/// What a form needs from the place it is filled in.
pub trait Console {
    /// Shows `prompt` and reads one line of text, without its line ending; `None` when the input has ended.
    ///
    /// # Errors
    ///
    /// Returns why the input could not be read.
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error>;

    /// Shows `prompt` and reads one line without showing it, as a password is typed; `None` when the user cancels
    /// or the input has ended.
    ///
    /// # Errors
    ///
    /// Returns why the input could not be read.
    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error>;

    /// Shows a message about the last input, such as why it was refused.
    ///
    /// # Errors
    ///
    /// Returns why the message could not be shown.
    fn report(&mut self, message: &str) -> Result<(), Error>;

    /// Lets the user make `choice`, and gives the indexes of the options chosen: one for [`Mode::Single`], one or
    /// more for [`Mode::Multiple`]. `None` when the user cancels.
    ///
    /// # Errors
    ///
    /// Returns why the choice could not be read.
    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error>;

    /// Whether someone answers at this console, so a form may offer to review its answers; `true` unless a console
    /// says otherwise.
    fn is_interactive(&self) -> bool {
        true
    }
}
