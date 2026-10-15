//! The yes-or-no question.

use super::{Common, Field, Kind};
use crate::form::Answers;

/// A yes-or-no question, whose value is a `bool`.
///
/// ```
/// use form_builder::Confirm;
///
/// let newsletter = Confirm::new("newsletter", "Subscribe to the newsletter?").default(false);
/// ```
#[must_use]
pub struct Confirm {
    common: Common,
    default: Option<bool>,
}

impl Confirm {
    /// A question with no default: the answer must be given.
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            default: None,
        }
    }

    /// The answer an empty input gives, shown as `[Y/n]` or `[y/N]`.
    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    common_methods!();
}

impl From<Confirm> for Field {
    fn from(confirm: Confirm) -> Self {
        Self::new(
            confirm.common,
            Kind::Confirm {
                default: confirm.default,
            },
        )
    }
}
