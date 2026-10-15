//! The form: built from fields by [`FormBuilder`], filled through a [`Console`], then read with [`Form::get`].

use std::any::type_name;
use std::fmt::Display;
use std::str::FromStr;

use crate::Error;
use crate::console::{Choice, Console, Terminal};
use crate::fields::{ContextRule, Fallback, Field, Kind, MultiSelect, Select, State, Text, Value};
use crate::prompt::{Messages, Prompt};
use crate::validate::Validator;

/// Collects the fields of a form, in the order they are asked.
///
/// The prompt of every required field is marked with `*`: `Name:` is asked as `Name *:`.
/// [`required_marker`](Self::required_marker) changes the mark, and
/// [`no_required_marker`](Self::no_required_marker) leaves it out.
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
    required_marker: Option<String>,
    messages: Messages,
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            required_marker: Some("*".to_owned()),
            messages: Messages::default(),
        }
    }
}

impl FormBuilder {
    /// A builder with no fields, which marks required fields with `*` and speaks English.
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
        self.required_marker = Some(marker.into());
        self
    }

    /// Leaves required fields unmarked, their prompts as given.
    pub fn no_required_marker(mut self) -> Self {
        self.required_marker = None;
        self
    }

    /// Replaces the texts the form shows of its own, such as the message of an invalid input.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
        self
    }

    /// The form, ready to be filled.
    #[must_use]
    pub fn build(self) -> Form {
        Form {
            fields: self.fields,
            required_marker: self.required_marker,
            messages: self.messages,
        }
    }
}

/// A form of named fields, filled in order, whose values are then read by name and type.
#[derive(Debug)]
pub struct Form {
    fields: Vec<Field>,
    required_marker: Option<String>,
    messages: Messages,
}

impl Form {
    /// Asks every field in the terminal, in order.
    ///
    /// # Errors
    ///
    /// Returns why a field could not be filled: the input ended, the user cancelled, a choice has no options, or
    /// the terminal failed.
    pub fn fill(&mut self) -> Result<(), Error> {
        self.fill_with(&mut Terminal)
    }

    /// Asks every field through `console`, in order, and skips a field whose condition does not hold for the answers
    /// before it.
    ///
    /// # Errors
    ///
    /// As [`fill`](Self::fill).
    pub fn fill_with(&mut self, console: &mut impl Console) -> Result<(), Error> {
        for index in 0..self.fields.len() {
            let (before, rest) = self.fields.split_at_mut(index);
            let field = &mut rest[0];
            let answers = Answers { fields: before };
            if field.condition.as_ref().is_some_and(|condition| !condition(&answers)) {
                field.state = State::Skipped;
                continue;
            }
            let asking = Asking {
                field,
                answers: &answers,
                prompt: prompt(field, self.required_marker.as_deref()),
                messages: &self.messages,
            };
            let value = asking.ask(console)?;
            field.state = State::Answered(value);
        }
        Ok(())
    }

    /// The value of the field `name`: `T` for a text field or a select, `Option<T>` for an optional text field,
    /// `Vec<T>` for a multiselect, `bool` for a confirm and `String` for a password.
    ///
    /// # Errors
    ///
    /// As [`Answers::get`].
    pub fn get<T: Clone + 'static>(&self, name: &str) -> Result<T, Error> {
        self.answers().get(name)
    }

    /// The answers given so far, as the conditions and rules of the fields see them.
    #[must_use]
    pub fn answers(&self) -> Answers<'_> {
        Answers { fields: &self.fields }
    }
}

/// The answers of a form so far: what a field's condition or rule sees of the fields before it.
#[derive(Debug, Clone, Copy)]
pub struct Answers<'a> {
    fields: &'a [Field],
}

impl Answers<'_> {
    /// The value of the field `name`, as [`Form::get`] gives it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when there is no such field, [`Error::NotFilled`] before it is asked,
    /// [`Error::Skipped`] when its condition kept it from being asked, and [`Error::WrongType`] when it holds another
    /// type.
    pub fn get<T: Clone + 'static>(&self, name: &str) -> Result<T, Error> {
        let field = self
            .fields
            .iter()
            .find(|field| field.name == name)
            .ok_or_else(|| Error::NotFound(name.to_owned()))?;
        let value = match &field.state {
            State::Answered(value) => value,
            State::Pending => return Err(Error::NotFilled(name.to_owned())),
            State::Skipped => return Err(Error::Skipped(name.to_owned())),
        };
        value.downcast_ref::<T>().cloned().ok_or_else(|| Error::WrongType {
            name: name.to_owned(),
            expected: type_name::<T>(),
        })
    }

    /// Whether the field `name` was answered with `value`: `answers.is("company", &true)`.
    #[must_use]
    pub fn is<T: PartialEq + 'static>(&self, name: &str, value: &T) -> bool {
        self.fields.iter().any(|field| {
            field.name == name
                && matches!(&field.state, State::Answered(answer) if answer.downcast_ref::<T>() == Some(value))
        })
    }

    /// Whether the field `name` has been answered.
    #[must_use]
    pub fn is_answered(&self, name: &str) -> bool {
        self.fields
            .iter()
            .any(|field| field.name == name && matches!(field.state, State::Answered(_)))
    }
}

/// The prompt `field` is asked with: its text, the mark of a required field, its hint and its default.
fn prompt(field: &Field, marker: Option<&str>) -> Prompt {
    let mut prompt = Prompt::new(&field.prompt);
    prompt.marker = marker.filter(|_| field.is_required()).map(str::to_owned);
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

/// One field being asked, with what it is asked with.
struct Asking<'a> {
    field: &'a Field,
    answers: &'a Answers<'a>,
    prompt: Prompt,
    messages: &'a Messages,
}

impl Asking<'_> {
    fn ask(&self, console: &mut impl Console) -> Result<Value, Error> {
        match &self.field.kind {
            Kind::Text {
                validator,
                rules,
                parse,
                fallback,
                optional,
            } => self.text(console, validator.as_ref(), rules, parse, fallback.as_ref(), *optional),
            Kind::Secret {
                validator,
                rules,
                repeat,
            } => self.secret(console, validator.as_ref(), rules, repeat.as_deref()),
            Kind::Confirm { default } => self.confirm(console, *default),
            Kind::Choice {
                options,
                mode,
                defaults,
                pick,
            } => {
                if options.is_empty() {
                    return Err(Error::NoOptions(self.field.name.clone()));
                }
                let choice = Choice {
                    prompt: &self.prompt,
                    options,
                    mode: *mode,
                    defaults,
                    messages: self.messages,
                };
                let chosen = console.choose(&choice)?.ok_or_else(|| self.cancelled())?;
                Ok(pick(&chosen))
            }
        }
    }

    fn text(
        &self,
        console: &mut impl Console,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        parse: &dyn Fn(&str) -> Result<Value, String>,
        fallback: Option<&Fallback>,
        optional: bool,
    ) -> Result<Value, Error> {
        loop {
            let line = console.read_line(&self.prompt)?.ok_or_else(|| self.ended())?;
            let input = line.trim();
            if input.is_empty() {
                if let Some(fallback) = fallback {
                    return Ok((fallback.value)());
                }
                if optional {
                    return parse(input).map_err(|_| self.ended());
                }
                console.report(&self.messages.required)?;
                continue;
            }
            if let Some(message) = self.refusal(validator, rules, input) {
                console.report(message)?;
                continue;
            }
            match parse(input) {
                Ok(value) => return Ok(value),
                Err(reason) => console.report(&format!("{}: {reason}", self.messages.invalid_input))?,
            }
        }
    }

    fn secret(
        &self,
        console: &mut impl Console,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        repeat: Option<&str>,
    ) -> Result<Value, Error> {
        loop {
            let first = console.read_secret(&self.prompt)?.ok_or_else(|| self.cancelled())?;
            if first.is_empty() {
                console.report(&self.messages.required)?;
                continue;
            }
            if let Some(message) = self.refusal(validator, rules, &first) {
                console.report(message)?;
                continue;
            }
            if let Some(repeat) = repeat {
                let mut again = Prompt::new(repeat);
                again.marker.clone_from(&self.prompt.marker);
                let second = console.read_secret(&again)?.ok_or_else(|| self.cancelled())?;
                if second != first {
                    console.report(&self.messages.passwords_differ)?;
                    continue;
                }
            }
            return Ok(Box::new(first));
        }
    }

    fn confirm(&self, console: &mut impl Console, default: Option<bool>) -> Result<Value, Error> {
        let (yes, no) = (initial(&self.messages.yes), initial(&self.messages.no));
        let mut prompt = self.prompt.clone();
        prompt.default = Some(match default {
            Some(true) => format!("{}/{no}", yes.to_uppercase()),
            Some(false) => format!("{yes}/{}", no.to_uppercase()),
            None => format!("{yes}/{no}"),
        });
        loop {
            let line = console.read_line(&prompt)?.ok_or_else(|| self.ended())?;
            let input = line.trim().to_lowercase();
            let answer = if input.is_empty() {
                default
            } else if self.messages.yes.to_lowercase().starts_with(&input) {
                Some(true)
            } else if self.messages.no.to_lowercase().starts_with(&input) {
                Some(false)
            } else {
                None
            };
            match answer {
                Some(answer) => return Ok(Box::new(answer)),
                None => console.report(&self.messages.answer_yes_or_no)?,
            }
        }
    }

    /// The message of the first check `input` fails: the validator's, then the rules that see earlier answers.
    fn refusal<'r>(&self, validator: Option<&'r Validator>, rules: &'r [ContextRule], input: &str) -> Option<&'r str> {
        if let Some(Err(message)) = validator.map(|validator| validator.check(input)) {
            return Some(message);
        }
        rules
            .iter()
            .find(|rule| !(rule.check)(self.answers, input))
            .map(|rule| rule.message.as_str())
    }

    fn ended(&self) -> Error {
        Error::EndOfInput(self.field.name.clone())
    }

    fn cancelled(&self) -> Error {
        Error::Cancelled(self.field.name.clone())
    }
}

/// The first letter of `word`, as the short answer of a yes-or-no question.
fn initial(word: &str) -> String {
    word.chars()
        .next()
        .map(|c| c.to_lowercase().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::initial;

    #[test]
    fn the_short_answer_is_the_first_letter() {
        assert_eq!(initial("Yes"), "y");
        assert_eq!(initial("tak"), "t");
        assert_eq!(initial(""), "");
    }
}
