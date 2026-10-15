//! The password, typed unseen.

use super::{Common, ContextRule, Field, Kind};
use crate::form::Answers;
use crate::validate::Validator;

/// A field read without showing what is typed, whose value is a `String`.
///
/// ```
/// use form_builder::Password;
/// use form_builder::validate::{self, Validator};
///
/// let password = Password::new("password", "Password:")
///     .validator(Validator::new().rule(validate::min_length(12), "At least 12 characters"))
///     .repeat("Repeat the password:");
/// ```
#[must_use]
pub struct Password {
    common: Common,
    validator: Option<Validator>,
    rules: Vec<ContextRule>,
    repeat: Option<String>,
}

impl Password {
    /// A password field: an empty answer is refused.
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            validator: None,
            rules: Vec::new(),
            repeat: None,
        }
    }

    /// Asks for the password a second time, with `prompt`, and asks again from the start when the two differ.
    pub fn repeat(mut self, prompt: impl Into<String>) -> Self {
        self.repeat = Some(prompt.into());
        self
    }

    common_methods!();
    check_methods!();
}

impl From<Password> for Field {
    fn from(password: Password) -> Self {
        let kind = Kind::Secret {
            validator: password.validator,
            rules: password.rules,
            repeat: password.repeat,
        };
        Self::new(password.common, kind)
    }
}
