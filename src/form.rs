//! The form: filled through a [`Console`], then read with [`Form::get`].

use std::any::type_name;

use crate::Error;
use crate::asking::{Asking, Outcome};
use crate::builder::Settings;
use crate::console::{Choice, Console};
use crate::fields::{Field, State};
use crate::prompt::Prompt;
use crate::selection::Mode;
use crate::terminal::Terminal;

/// A form of named fields, filled in order, whose values are then read by name and type.
#[derive(Debug)]
pub struct Form {
    fields: Vec<Field>,
    settings: Settings,
}

impl Form {
    pub(crate) fn new(fields: Vec<Field>, settings: Settings) -> Self {
        Self { fields, settings }
    }

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

    /// Goes through the fields from `start` on: skips those whose condition does not hold, keeps those answered,
    /// and asks the others.
    fn ask_from(&mut self, start: usize, console: &mut impl Console) -> Result<(), Error> {
        for index in start..self.fields.len() {
            if !self.condition_holds(index) {
                self.fields[index].state = State::Skipped;
            } else if !matches!(self.fields[index].state, State::Answered { .. }) {
                self.ask(index, false, console)?;
            }
        }
        Ok(())
    }

    /// Whether the field at `index` is to be asked, given the answers before it.
    fn condition_holds(&self, index: usize) -> bool {
        let (before, rest) = self.fields.split_at(index);
        let answers = Answers { fields: before };
        rest[0].condition.as_ref().is_none_or(|condition| condition(&answers))
    }

    /// Asks the field at `index`, as a change of its answer from the review when `editing`, and gives whether it
    /// took a new answer.
    fn ask(&mut self, index: usize, editing: bool, console: &mut impl Console) -> Result<bool, Error> {
        let len = self.fields.len();
        let (before, rest) = self.fields.split_at_mut(index);
        let field = &mut rest[0];
        let answers = Answers { fields: before };
        let asking = Asking {
            field,
            answers: &answers,
            prompt: self.settings.prompt(field, index, len),
            messages: &self.settings.messages,
            editing,
        };
        match asking.ask(console)? {
            Outcome::Answered(answer) => {
                field.state = State::Answered {
                    value: answer.value,
                    shown: answer.shown,
                    chosen: answer.chosen,
                };
                Ok(true)
            }
            Outcome::Kept => Ok(false),
        }
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
                back: false,
            };
            let chosen = console
                .choose(&choice)?
                .ok_or_else(|| Error::Cancelled("review".to_owned()))?;
            let Some(&index) = answered.get(chosen[0]) else {
                return Ok(());
            };
            if self.ask(index, true, console)? {
                self.ask_from(index + 1, console)?;
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
