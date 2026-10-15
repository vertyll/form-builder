//! Answers given ahead of time: a [`Console`] that takes them from a list, the environment or the command line, so
//! the same form runs where nobody can answer it, as in CI or a script.

use std::collections::HashMap;
use std::fmt;

use crate::Error;
use crate::console::{Choice, Console};
use crate::prompt::Prompt;
use crate::selection::Mode;

/// A console that answers each field by its name from answers given ahead of time.
///
/// A field it has no answer for goes to its fallback console, such as the [`Terminal`](crate::Terminal); with none,
/// the form ends with [`Error::EndOfInput`]. An answer the form refuses ends it with [`Error::Refused`] and the reason,
/// or, with a fallback, is asked there again. A select takes the label of an option, a multiselect labels separated
/// by commas, and a yes-or-no question the form's words for yes and no.
///
/// ```no_run
/// use form_builder::{Error, FormBuilder, Preset, Terminal, Text};
///
/// fn main() -> Result<(), Error> {
///     let mut form = FormBuilder::new().field(Text::<u16>::new("port", "Port:")).build();
///     let mut console = Preset::from_env("APP_")
///         .merge(Preset::from_args(std::env::args().skip(1)))
///         .or(Terminal);
///     form.fill_with(&mut console)?;
///     Ok(())
/// }
/// ```
#[derive(Default)]
#[must_use]
pub struct Preset {
    answers: HashMap<String, String>,
    fallback: Option<Box<dyn Console + Send>>,
    last: Option<String>,
    refused: HashMap<String, String>,
}

impl Preset {
    /// The answers `answers` gives, each a field's name and its answer.
    pub fn new<K: Into<String>, V: Into<String>>(answers: impl IntoIterator<Item = (K, V)>) -> Self {
        Self {
            answers: answers
                .into_iter()
                .map(|(name, value)| (name.into(), value.into()))
                .collect(),
            ..Self::default()
        }
    }

    /// The answers in the environment variables that start with `prefix`: `APP_PORT=8080` answers the field `port`
    /// for the prefix `APP_`.
    pub fn from_env(prefix: &str) -> Self {
        Self::from_vars(prefix, std::env::vars())
    }

    /// The answers in `vars`, named as [`from_env`](Self::from_env) names them.
    pub fn from_vars(prefix: &str, vars: impl IntoIterator<Item = (String, String)>) -> Self {
        Self::new(vars.into_iter().filter_map(|(key, value)| {
            let name = key.strip_prefix(prefix)?.to_lowercase();
            (!name.is_empty()).then_some((name, value))
        }))
    }

    /// The answers in `args`, each `name=value` or `--name=value`; other arguments are left out.
    pub fn from_args<A: AsRef<str>>(args: impl IntoIterator<Item = A>) -> Self {
        Self::new(args.into_iter().filter_map(|arg| {
            let arg = arg.as_ref();
            let (name, value) = arg.trim_start_matches("--").split_once('=')?;
            (!name.is_empty()).then(|| (name.to_owned(), value.to_owned()))
        }))
    }

    /// Adds the answer `value` for the field `name`, in place of any it had.
    pub fn answer(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.answers.insert(name.into(), value.into());
        self
    }

    /// Adds the answers of `other`, which take the place of those of the same fields.
    pub fn merge(mut self, other: Self) -> Self {
        self.answers.extend(other.answers);
        self
    }

    /// Asks `console` for the fields it has no answer for, and again for one whose answer was refused.
    pub fn or(mut self, console: impl Console + Send + 'static) -> Self {
        self.fallback = Some(Box::new(console));
        self
    }

    /// The preset answer of the field `name`, which is taken only once.
    fn take(&mut self, name: &str) -> Option<String> {
        let answer = self.answers.remove(name)?;
        self.last = Some(name.to_owned());
        Some(answer)
    }

    /// Checks the field `name`, which has no preset answer left, may go on: with no fallback to ask, a refused answer
    /// ends the form with why it was refused.
    fn missing(&mut self, name: &str) -> Result<(), Error> {
        self.last = None;
        match self.refused.remove(name) {
            Some(message) if self.fallback.is_none() => Err(Error::Refused {
                name: name.to_owned(),
                message,
            }),
            _ => Ok(()),
        }
    }

    /// The indexes of the options `answer` names by their labels, or why it names none of them rightly.
    fn options(answer: &str, choice: &Choice<'_>) -> Result<Vec<usize>, String> {
        let find = |label: &str| {
            choice
                .options
                .iter()
                .position(|option| option.eq_ignore_ascii_case(label.trim()))
                .ok_or_else(|| format!("no option named `{}`", label.trim()))
        };
        match choice.mode {
            Mode::Single => Ok(vec![find(answer)?]),
            Mode::Multiple { limit } => {
                let chosen = answer.split(',').map(find).collect::<Result<Vec<_>, _>>()?;
                if chosen.is_empty() || limit.is_some_and(|limit| chosen.len() > limit) {
                    return Err(format!(
                        "choose between 1 and {} options",
                        limit.unwrap_or(choice.options.len())
                    ));
                }
                Ok(chosen)
            }
        }
    }
}

impl Console for Preset {
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        if let Some(answer) = self.take(&prompt.name) {
            return Ok(Some(answer));
        }
        self.missing(&prompt.name)?;
        match &mut self.fallback {
            Some(fallback) => fallback.read_line(prompt),
            None => Ok(None),
        }
    }

    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        if let Some(answer) = self.take(&prompt.name) {
            return Ok(Some(answer));
        }
        self.missing(&prompt.name)?;
        match &mut self.fallback {
            Some(fallback) => fallback.read_secret(prompt),
            None => Ok(None),
        }
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        if let Some(name) = self.last.take() {
            self.refused.insert(name, message.to_owned());
        }
        match &mut self.fallback {
            Some(fallback) => fallback.report(message),
            None => Ok(()),
        }
    }

    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error> {
        let name = &choice.prompt.name;
        if let Some(answer) = self.take(name) {
            self.last = None;
            match Self::options(&answer, choice) {
                Ok(chosen) => return Ok(Some(chosen)),
                Err(message) if self.fallback.is_none() => {
                    return Err(Error::Refused {
                        name: name.clone(),
                        message,
                    });
                }
                Err(message) => self.report(&message)?,
            }
        }
        match &mut self.fallback {
            Some(fallback) => fallback.choose(choice),
            None => Ok(None),
        }
    }

    fn is_interactive(&self) -> bool {
        self.fallback.as_ref().is_some_and(|fallback| fallback.is_interactive())
    }
}

impl fmt::Debug for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names: Vec<&String> = self.answers.keys().collect();
        names.sort();
        f.debug_struct("Preset")
            .field("answers", &names)
            .field("fallback", &self.fallback.is_some())
            .finish_non_exhaustive()
    }
}
