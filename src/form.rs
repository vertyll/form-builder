//! The form: built from fields by [`FormBuilder`], filled through a [`Console`], then read with [`Form::get`].

use std::any::type_name;
use std::fmt::Display;
use std::str::FromStr;

use crate::Error;
use crate::console::{Choice, Console, Terminal};
use crate::fields::{ContextRule, Fallback, Field, Kind, MultiSelect, Parse, Refusal, Select, State, Text, Value};
use crate::prompt::{Messages, Prompt};
use crate::selection::Mode;
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
struct Settings {
    required_marker: Option<String>,
    steps: bool,
    review: bool,
    messages: Messages,
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
    /// [interactive](Console::is_interactive) skips the review.
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
        Form {
            fields: self.fields,
            settings: self.settings,
        }
    }
}

/// A form of named fields, filled in order, whose values are then read by name and type.
#[derive(Debug)]
pub struct Form {
    fields: Vec<Field>,
    settings: Settings,
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
    /// before it; then, when asked for, reviews the answers.
    ///
    /// # Errors
    ///
    /// As [`fill`](Self::fill).
    pub fn fill_with(&mut self, console: &mut impl Console) -> Result<(), Error> {
        self.ask_from(0, console)?;
        if self.settings.review && console.is_interactive() {
            self.review(console)?;
        }
        Ok(())
    }

    /// Asks the fields from `start` on whose condition holds, and skips the others; a field already answered is
    /// asked again only when it is `start` itself.
    fn ask_from(&mut self, start: usize, console: &mut impl Console) -> Result<(), Error> {
        let len = self.fields.len();
        for index in start..len {
            let (before, rest) = self.fields.split_at_mut(index);
            let field = &mut rest[0];
            let answers = Answers { fields: before };
            if field.condition.as_ref().is_some_and(|condition| !condition(&answers)) {
                field.state = State::Skipped;
                continue;
            }
            if index != start && matches!(field.state, State::Answered { .. }) {
                continue;
            }
            let asking = Asking {
                field,
                answers: &answers,
                prompt: self.settings.prompt(field, index, len),
                messages: &self.settings.messages,
            };
            let (value, shown) = asking.ask(console)?;
            field.state = State::Answered { value, shown };
        }
        Ok(())
    }

    /// Lists the answers until the user submits them, asking again the field they choose.
    fn review(&mut self, console: &mut impl Console) -> Result<(), Error> {
        loop {
            let answered: Vec<usize> = (0..self.fields.len())
                .filter(|&index| matches!(self.fields[index].state, State::Answered { .. }))
                .collect();
            let mut options: Vec<String> = answered.iter().map(|&index| self.summary(index)).collect();
            options.push(self.settings.messages.submit.clone());
            let prompt = Prompt::new("review", &self.settings.messages.review);
            let choice = Choice {
                prompt: &prompt,
                options: &options,
                mode: Mode::Single,
                defaults: &[answered.len()],
                messages: &self.settings.messages,
            };
            let chosen = console
                .choose(&choice)?
                .ok_or_else(|| Error::Cancelled("review".to_owned()))?;
            match answered.get(chosen[0]) {
                Some(&index) => self.ask_from(index, console)?,
                None => return Ok(()),
            }
        }
    }

    /// The line of the review for the field at `index`: its label and its answer.
    fn summary(&self, index: usize) -> String {
        let field = &self.fields[index];
        let label = Prompt::new(&field.name, &field.prompt).label;
        let shown = match &field.state {
            State::Answered { shown, .. } if shown.is_empty() => self.settings.messages.empty_answer.as_str(),
            State::Answered { shown, .. } => shown.as_str(),
            State::Pending | State::Skipped => "",
        };
        format!("{label}: {shown}")
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
            State::Answered { value, .. } => value,
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
                && matches!(&field.state, State::Answered { value: answer, .. } if answer.downcast_ref::<T>() == Some(value))
        })
    }

    /// Whether the field `name` has been answered.
    #[must_use]
    pub fn is_answered(&self, name: &str) -> bool {
        self.fields
            .iter()
            .any(|field| field.name == name && matches!(field.state, State::Answered { .. }))
    }
}

impl Settings {
    /// The prompt the field at `index`, of `len`, is asked with: its step, its text, the mark of a required field, its
    /// hint and its default.
    fn prompt(&self, field: &Field, index: usize, len: usize) -> Prompt {
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

/// One field being asked, with what it is asked with.
struct Asking<'a> {
    field: &'a Field,
    answers: &'a Answers<'a>,
    prompt: Prompt,
    messages: &'a Messages,
}

/// What a field gave: its value, and the answer as the review shows it.
type Answer = (Value, String);

impl Asking<'_> {
    fn ask(&self, console: &mut impl Console) -> Result<Answer, Error> {
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
                let chosen = console.choose(&choice)?.ok_or_else(|| self.unanswered(console))?;
                let shown: Vec<&str> = chosen.iter().map(|&index| options[index].as_str()).collect();
                Ok((pick(&chosen), shown.join(", ")))
            }
        }
    }

    fn text(
        &self,
        console: &mut impl Console,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        parse: &Parse,
        fallback: Option<&Fallback>,
        optional: bool,
    ) -> Result<Answer, Error> {
        loop {
            let line = self.line(console, &self.prompt, fallback.is_some() || optional)?;
            let input = line.trim();
            let outcome = if input.is_empty() {
                self.empty(parse, fallback, optional)
            } else {
                self.take(input, validator, rules, parse)
            };
            match outcome {
                Ok(answer) => return Ok(answer),
                Err(message) => console.report(&message)?,
            }
        }
    }

    /// Reads a line at `prompt`, or, at a console nobody answers, takes an empty one when the field `takes_empty`.
    fn line(&self, console: &mut impl Console, prompt: &Prompt, takes_empty: bool) -> Result<String, Error> {
        match console.read_line(prompt)? {
            Some(line) => Ok(line),
            None if takes_empty && !console.is_interactive() => Ok(String::new()),
            None => Err(self.ended()),
        }
    }

    /// What an empty answer gives: the default, `None` for an optional field, or the message of a required one.
    fn empty(&self, parse: &Parse, fallback: Option<&Fallback>, optional: bool) -> Result<Answer, String> {
        if let Some(fallback) = fallback {
            return Ok(((fallback.value)(), fallback.shown.clone()));
        }
        if optional && let Ok(value) = parse("") {
            return Ok((value, String::new()));
        }
        Err(self.messages.required.clone())
    }

    /// The value `input` gives, or the message of the first check it fails.
    fn take(
        &self,
        input: &str,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        parse: &Parse,
    ) -> Result<Answer, String> {
        if let Some(message) = self.refusal(validator, rules, input) {
            return Err(message.to_owned());
        }
        match parse(input) {
            Ok(value) => Ok((value, input.to_owned())),
            Err(Refusal::Unparsed(reason)) => Err(format!("{}: {reason}", self.messages.invalid_input)),
            Err(Refusal::Checked(message)) => Err(message),
        }
    }

    fn secret(
        &self,
        console: &mut impl Console,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        repeat: Option<&str>,
    ) -> Result<Answer, Error> {
        loop {
            let first = console
                .read_secret(&self.prompt)?
                .ok_or_else(|| self.unanswered(console))?;
            if first.is_empty() {
                console.report(&self.messages.required)?;
                continue;
            }
            if let Some(message) = self.refusal(validator, rules, &first) {
                console.report(message)?;
                continue;
            }
            if let Some(repeat) = repeat.filter(|_| console.is_interactive()) {
                let mut again = Prompt::new(&self.field.name, repeat);
                again.marker.clone_from(&self.prompt.marker);
                let second = console.read_secret(&again)?.ok_or_else(|| self.unanswered(console))?;
                if second != first {
                    console.report(&self.messages.passwords_differ)?;
                    continue;
                }
            }
            return Ok((Box::new(first), self.messages.hidden_answer.clone()));
        }
    }

    fn confirm(&self, console: &mut impl Console, default: Option<bool>) -> Result<Answer, Error> {
        let (yes, no) = (initial(&self.messages.yes), initial(&self.messages.no));
        let mut prompt = self.prompt.clone();
        prompt.default = Some(match default {
            Some(true) => format!("{}/{no}", yes.to_uppercase()),
            Some(false) => format!("{yes}/{}", no.to_uppercase()),
            None => format!("{yes}/{no}"),
        });
        loop {
            let line = self.line(console, &prompt, default.is_some())?;
            match self.yes_or_no(&line, default) {
                Some(answer) => {
                    let word = if answer { &self.messages.yes } else { &self.messages.no };
                    return Ok((Box::new(answer), word.clone()));
                }
                None => console.report(&self.messages.answer_yes_or_no)?,
            }
        }
    }

    /// The answer `line` gives to a yes-or-no question: the default when it is empty, yes or no when it starts one of
    /// the form's words for them, or none.
    fn yes_or_no(&self, line: &str, default: Option<bool>) -> Option<bool> {
        let input = line.trim().to_lowercase();
        if input.is_empty() {
            default
        } else if self.messages.yes.to_lowercase().starts_with(&input) {
            Some(true)
        } else if self.messages.no.to_lowercase().starts_with(&input) {
            Some(false)
        } else {
            None
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

    /// The error of a choice or a password left without an answer: cancelled by the user at an interactive console,
    /// or with no answer to give at another, as a [`Preset`](crate::Preset) without one.
    fn unanswered(&self, console: &impl Console) -> Error {
        if console.is_interactive() {
            Error::Cancelled(self.field.name.clone())
        } else {
            self.ended()
        }
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
