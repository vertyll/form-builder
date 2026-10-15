//! Asking one field: reading its answer through a console until the field takes it.

use crate::Error;
use crate::console::{Choice, Console};
use crate::fields::{ContextRule, Fallback, Field, Kind, Parse, Refusal, Value};
use crate::form::Answers;
use crate::prompt::{Messages, Prompt};
use crate::validate::Validator;

/// One field being asked, with what it is asked with.
pub(crate) struct Asking<'a> {
    pub(crate) field: &'a Field,
    pub(crate) answers: &'a Answers<'a>,
    pub(crate) prompt: Prompt,
    pub(crate) messages: &'a Messages,
}

/// What a field gave: its value, and the answer as the review shows it.
pub(crate) type Answer = (Value, String);

impl Asking<'_> {
    pub(crate) fn ask(&self, console: &mut impl Console) -> Result<Answer, Error> {
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
