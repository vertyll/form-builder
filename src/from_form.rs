//! A type filled from a form: what `#[derive(Form)]` implements.

use crate::Error;
use crate::builder::FormBuilder;
use crate::console::Console;
use crate::form::Form;
use crate::terminal::Terminal;

/// A type whose values a form asks for, field by field, as `#[derive(Form)]` writes it with the `derive` feature.
///
/// ```ignore
/// use form_builder::{Form, FromForm};
///
/// #[derive(Form)]
/// struct SignUp {
///     name: String,
///     #[form(prompt = "Port:", default = 8080, hint = "1024-65535")]
///     port: u16,
///     #[form(password, repeat = "Repeat the password:")]
///     password: String,
///     newsletter: bool,
///     company: Option<String>,
/// }
///
/// let sign_up = SignUp::fill()?;
/// ```
pub trait FromForm: Sized {
    /// The builder of the form that asks for the type's values, to adjust before it is built, as with
    /// [`FormBuilder::review`].
    fn form() -> FormBuilder;

    /// The value the answers of `form` make.
    ///
    /// # Errors
    ///
    /// Returns why a field could not be read, as [`Form::get`] does.
    fn from_form(form: &Form) -> Result<Self, Error>;

    /// Asks for the value in the terminal.
    ///
    /// # Errors
    ///
    /// Returns why the form could not be filled or read.
    fn fill() -> Result<Self, Error> {
        Self::fill_with(&mut Terminal)
    }

    /// Asks for the value through `console`.
    ///
    /// # Errors
    ///
    /// Returns why the form could not be filled or read.
    fn fill_with(console: &mut impl Console) -> Result<Self, Error> {
        let mut form = Self::form().build();
        form.fill_with(console)?;
        Self::from_form(&form)
    }
}
