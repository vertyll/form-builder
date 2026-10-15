//! The text field: input parsed into a type, required or optional, with a default.

use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use super::{Common, ContextRule, Fallback, Field, Kind, Parse, Refusal};
use crate::form::Answers;
use crate::validate::Validator;

/// A check of a parsed value, and the message shown when it fails.
type ValueCheck<T> = (Box<dyn Fn(&T) -> bool + Send + Sync>, String);

/// A field read as text and parsed into a `T`, or into an `Option<T>` when [optional](Self::optional).
///
/// ```
/// use form_builder::Text;
///
/// let port = Text::<u16>::new("port", "Port:")
///     .default(8080)
///     .hint("1024-65535")
///     .check(|port| *port >= 1024, "Use a port above 1023");
/// ```
#[must_use]
pub struct Text<T> {
    common: Common,
    validator: Option<Validator>,
    rules: Vec<ContextRule>,
    checks: Vec<ValueCheck<T>>,
    fallback: Option<Fallback>,
    optional: bool,
    value: PhantomData<fn() -> T>,
}

impl<T> Text<T>
where
    T: FromStr + Send + Sync + 'static,
    T::Err: Display,
{
    /// A required field: an empty answer is refused unless the field has a [default](Self::default).
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            validator: None,
            rules: Vec::new(),
            checks: Vec::new(),
            fallback: None,
            optional: false,
            value: PhantomData,
        }
    }

    /// A field that may be left empty: its value is an `Option<T>`, `None` for an empty answer, which skips the
    /// checks.
    pub fn optional(name: &str, prompt: &str) -> Self {
        Self {
            optional: true,
            ..Self::new(name, prompt)
        }
    }

    /// Checks the value once it is parsed, such as a number's range: `message` is shown when `check` fails, and
    /// the field asks again. A [default](Self::default) is taken as it is.
    pub fn check(mut self, check: impl Fn(&T) -> bool + Send + Sync + 'static, message: impl Into<String>) -> Self {
        self.checks.push((Box::new(check), message.into()));
        self
    }

    common_methods!();
    check_methods!();
}

impl<T> Text<T>
where
    T: FromStr + Clone + Display + Send + Sync + 'static,
    T::Err: Display,
{
    /// The value an empty answer gives, shown with the prompt as `[value]`; for an optional field, `Some(value)`.
    pub fn default(mut self, value: T) -> Self {
        let shown = value.to_string();
        let optional = self.optional;
        self.fallback = Some(Fallback {
            shown,
            value: Box::new(move || {
                if optional {
                    Box::new(Some(value.clone()))
                } else {
                    Box::new(value.clone())
                }
            }),
        });
        self
    }
}

impl<T> From<Text<T>> for Field
where
    T: FromStr + Send + Sync + 'static,
    T::Err: Display,
{
    fn from(text: Text<T>) -> Self {
        let optional = text.optional;
        let checks = text.checks;
        let parse: Parse = Box::new(move |input| {
            if optional && input.is_empty() {
                return Ok(Box::new(None::<T>));
            }
            let value = input
                .parse::<T>()
                .map_err(|error| Refusal::Unparsed(error.to_string()))?;
            if let Some((_, message)) = checks.iter().find(|(check, _)| !check(&value)) {
                return Err(Refusal::Checked(message.clone()));
            }
            Ok(if optional {
                Box::new(Some(value))
            } else {
                Box::new(value)
            })
        });
        let kind = Kind::Text {
            validator: text.validator,
            rules: text.rules,
            parse,
            fallback: text.fallback,
            optional,
        };
        Self::new(text.common, kind)
    }
}

impl<T> Text<T>
where
    T: FromStr + Send + Sync + 'static,
    T::Err: Display,
{
    pub(crate) fn validator_if(self, validator: Option<Validator>) -> Self {
        match validator {
            Some(validator) => self.validator(validator),
            None => self,
        }
    }
}
