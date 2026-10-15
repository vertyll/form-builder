//! The text field: input parsed into a type, required or optional, with a default.

use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use super::{Common, ContextRule, Fallback, Field, Kind, Parse};
use crate::form::Answers;
use crate::validate::Validator;

/// A field read as text and parsed into a `T`, or into an `Option<T>` when [optional](Self::optional).
///
/// ```
/// use form_builder::Text;
///
/// let port = Text::<u16>::new("port", "Port:").default(8080).hint("1-65535");
/// ```
#[must_use]
pub struct Text<T> {
    common: Common,
    validator: Option<Validator>,
    rules: Vec<ContextRule>,
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
        let parse: Parse = Box::new(move |input| match input.parse::<T>() {
            Ok(value) if optional => Ok(Box::new(Some(value))),
            Ok(value) => Ok(Box::new(value)),
            Err(_) if optional && input.is_empty() => Ok(Box::new(None::<T>)),
            Err(error) => Err(error.to_string()),
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
