//! The kinds of field a form asks, each built by its own type: [`Text`], [`Select`], [`MultiSelect`], [`Confirm`]
//! and [`Password`]. Each takes a hint shown with its prompt and a condition under which it is asked at all.

use std::any::Any;
use std::fmt;

use crate::form::Answers;
use crate::selection::Mode;
use crate::validate::Validator;

pub(crate) type Value = Box<dyn Any + Send + Sync>;
pub(crate) type Condition = Box<dyn Fn(&Answers<'_>) -> bool + Send + Sync>;
pub(crate) type Check = Box<dyn Fn(&Answers<'_>, &str) -> bool + Send + Sync>;
pub(crate) type Parse = Box<dyn Fn(&str) -> Result<Value, String> + Send + Sync>;
pub(crate) type Pick = Box<dyn Fn(&[usize]) -> Value + Send + Sync>;

/// A check of an input against the answers given before it, and the message shown when it fails.
pub(crate) struct ContextRule {
    pub(crate) check: Check,
    pub(crate) message: String,
}

/// What an empty answer to a text field gives: the default as shown, and the value itself.
pub(crate) struct Fallback {
    pub(crate) shown: String,
    pub(crate) value: Box<dyn Fn() -> Value + Send + Sync>,
}

/// What a field asks for, with what turns the answer into its value.
pub(crate) enum Kind {
    Text {
        validator: Option<Validator>,
        rules: Vec<ContextRule>,
        parse: Parse,
        fallback: Option<Fallback>,
        optional: bool,
    },
    Secret {
        validator: Option<Validator>,
        rules: Vec<ContextRule>,
        repeat: Option<String>,
    },
    Confirm {
        default: Option<bool>,
    },
    Choice {
        options: Vec<String>,
        mode: Mode,
        defaults: Vec<usize>,
        pick: Pick,
    },
}

/// Whether a field has been asked, and what it gave.
pub(crate) enum State {
    Pending,
    Skipped,
    Answered(Value),
}

/// A field of a form, made from a [`Text`], [`Select`], [`MultiSelect`], [`Confirm`] or [`Password`] and added with
/// [`FormBuilder::field`](crate::FormBuilder::field).
pub struct Field {
    pub(crate) name: String,
    pub(crate) prompt: String,
    pub(crate) hint: Option<String>,
    pub(crate) condition: Option<Condition>,
    pub(crate) kind: Kind,
    pub(crate) state: State,
}

impl Field {
    fn new(common: Common, kind: Kind) -> Self {
        Self {
            name: common.name,
            prompt: common.prompt,
            hint: common.hint,
            condition: common.condition,
            kind,
            state: State::Pending,
        }
    }

    /// Whether the field needs an answer: every one but an optional text field.
    pub(crate) fn is_required(&self) -> bool {
        !matches!(self.kind, Kind::Text { optional: true, .. })
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("prompt", &self.prompt)
            .finish_non_exhaustive()
    }
}

/// What every kind of field has: its name, prompt, hint and condition.
struct Common {
    name: String,
    prompt: String,
    hint: Option<String>,
    condition: Option<Condition>,
}

impl Common {
    fn new(name: &str, prompt: &str) -> Self {
        Self {
            name: name.to_owned(),
            prompt: prompt.to_owned(),
            hint: None,
            condition: None,
        }
    }
}

/// Adds the methods every kind of field has.
macro_rules! common_methods {
    () => {
        /// Shows `hint` with the prompt, as a short note on what to enter.
        pub fn hint(mut self, hint: impl Into<String>) -> Self {
            self.common.hint = Some(hint.into());
            self
        }

        /// Asks the field only when `condition` holds for the answers given before it; otherwise the field is
        /// skipped, and reading it gives [`Error::Skipped`](crate::Error::Skipped).
        pub fn when(mut self, condition: impl Fn(&Answers<'_>) -> bool + Send + Sync + 'static) -> Self {
            self.common.condition = Some(Box::new(condition));
            self
        }
    };
}

/// Adds the methods of the fields whose input is checked: a validator, and rules that see earlier answers.
macro_rules! check_methods {
    () => {
        /// Checks the trimmed input with `validator` before it is taken; a failed check shows its message, and the
        /// field asks again.
        pub fn validator(mut self, validator: Validator) -> Self {
            self.validator = Some(validator);
            self
        }

        /// Checks the input against the answers given before it, such as a range that depends on an earlier field;
        /// `message` is shown when `check` fails.
        pub fn rule_with(
            mut self,
            check: impl Fn(&Answers<'_>, &str) -> bool + Send + Sync + 'static,
            message: impl Into<String>,
        ) -> Self {
            self.rules.push(ContextRule {
                check: Box::new(check),
                message: message.into(),
            });
            self
        }
    };
}

mod confirm;
mod password;
mod select;
mod text;

pub use confirm::Confirm;
pub use password::Password;
pub use select::{MultiSelect, Select};
pub use text::Text;
