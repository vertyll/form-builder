//! The builder of a form: its fields, and how it shows them.

use std::fmt::Display;
use std::str::FromStr;

use crate::fields::{Fallback, Field, Kind, MultiSelect, Select, Text};
use crate::form::Form;
use crate::prompt::{Messages, Prompt};
use crate::validate::Validator;

/// Collects the fields of a form, in the order they are asked.
///
/// Every prompt starts with its step, `[3/8]`, which [`no_steps`](Self::no_steps) leaves out. The prompt of every
/// required field is marked with `*`: `Name:` is asked as `Name *:`. [`required_marker`](Self::required_marker)
/// changes the mark, and [`no_required_marker`](Self::no_required_marker) leaves it out. [`review`](Self::review)
/// lists the answers at the end, to change any before the form is done.
///
/// ```
/// use form_builder::{Confirm, FormBuilder, Select, Text};
///
/// let form = FormBuilder::new()
///     .field(Text::<String>::new("name", "Name:"))
///     .field(Text::<u16>::new("port", "Port:").default(8080))
///     .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&1))
///     .field(Confirm::new("newsletter", "Newsletter?").default(false))
///     .build();
/// ```
#[derive(Debug)]
#[must_use]
pub struct FormBuilder {
    fields: Vec<Field>,
    settings: Settings,
}

/// How a form shows its fields.
#[derive(Debug)]
pub(crate) struct Settings {
    pub(crate) required_marker: Option<String>,
    pub(crate) steps: bool,
    pub(crate) review: bool,
    pub(crate) messages: Messages,
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            settings: Settings {
                required_marker: Some("*".to_owned()),
                steps: true,
                review: false,
                messages: Messages::default(),
            },
        }
    }
}

impl FormBuilder {
    /// A builder with no fields, which shows steps, marks required fields with `*` and speaks English.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field: a [`Text`], [`Select`], [`MultiSelect`], [`Confirm`](crate::Confirm) or [`Password`](crate::Password).
    pub fn field(mut self, field: impl Into<Field>) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Adds a required [`Text`] field parsed into a `T`, checked by `validator` when given.
    pub fn add_field<T>(self, name: &str, prompt: &str, validator: Option<Validator>) -> Self
    where
        T: FromStr + Send + Sync + 'static,
        T::Err: Display,
    {
        self.field(Text::<T>::new(name, prompt).validator_if(validator))
    }

    /// Adds an [optional](Text::optional) [`Text`] field, whose value is an `Option<T>`.
    pub fn add_optional_field<T>(self, name: &str, prompt: &str, validator: Option<Validator>) -> Self
    where
        T: FromStr + Send + Sync + 'static,
        T::Err: Display,
    {
        self.field(Text::<T>::optional(name, prompt).validator_if(validator))
    }

    /// Adds a [`Select`] over `options`, each a value and the label shown for it.
    pub fn add_select<T, L>(self, name: &str, prompt: &str, options: impl IntoIterator<Item = (T, L)>) -> Self
    where
        T: Clone + Send + Sync + 'static,
        L: Into<String>,
    {
        self.field(Select::new(name, prompt, options))
    }

    /// Adds a [`MultiSelect`] over `options`, at most `limit` of them chosen when given.
    pub fn add_multiselect<T, L>(
        self,
        name: &str,
        prompt: &str,
        options: impl IntoIterator<Item = (T, L)>,
        limit: Option<usize>,
    ) -> Self
    where
        T: Clone + Send + Sync + 'static,
        L: Into<String>,
    {
        self.field(MultiSelect::new(name, prompt, options).limit_to(limit))
    }

    /// Marks every required field with `marker` instead of `*`. A field is required unless it is an
    /// [optional](Text::optional) text field.
    pub fn required_marker(mut self, marker: impl Into<String>) -> Self {
        self.settings.required_marker = Some(marker.into());
        self
    }

    /// Leaves required fields unmarked, their prompts as given.
    pub fn no_required_marker(mut self) -> Self {
        self.settings.required_marker = None;
        self
    }

    /// Leaves out the step, `[3/8]`, that starts every prompt.
    pub fn no_steps(mut self) -> Self {
        self.settings.steps = false;
        self
    }

    /// Lists the answers once every field is asked, and lets the user choose one to change before the form is done.
    /// A field asked again may change which fields after it are asked. A console that is not
    /// [interactive](crate::Console::is_interactive) skips the review.
    pub fn review(mut self) -> Self {
        self.settings.review = true;
        self
    }

    /// Replaces the texts the form shows of its own, such as the message of an invalid input.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.settings.messages = messages;
        self
    }

    /// The form, ready to be filled.
    #[must_use]
    pub fn build(self) -> Form {
        Form::new(self.fields, self.settings)
    }
}

impl Settings {
    /// The prompt the field at `index`, of `len`, is asked with: its step, its text, the mark of a required field, its
    /// hint and its default.
    pub(crate) fn prompt(&self, field: &Field, index: usize, len: usize) -> Prompt {
        let mut prompt = Prompt::new(&field.name, &field.prompt);
        prompt.step = self.steps.then_some((index + 1, len));
        prompt.marker = self
            .required_marker
            .as_deref()
            .filter(|_| field.is_required())
            .map(str::to_owned);
        prompt.hint.clone_from(&field.hint);
        if let Kind::Text {
            fallback: Some(Fallback { shown, .. }),
            ..
        } = &field.kind
        {
            prompt.default = Some(shown.clone());
        }
        prompt
    }
}
