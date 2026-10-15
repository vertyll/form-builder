//! Asking one field: reading its answer through a console until the field takes it, or, when the answer is being
//! changed from the review, until the user keeps the one it had.

use crate::Error;
use crate::console::{Choice, Console};
use crate::fields::{ContextRule, Fallback, Field, Kind, Parse, Refusal, State, Value};
use crate::form::Answers;
use crate::prompt::{Messages, Prompt};
use crate::validate::Validator;

/// One field being asked, with what it is asked with.
pub(crate) struct Asking<'a> {
    pub(crate) field: &'a Field,
    pub(crate) answers: &'a Answers<'a>,
    pub(crate) prompt: Prompt,
    pub(crate) messages: &'a Messages,
    /// Whether the field is asked again from the review, so its answer may be kept.
    pub(crate) editing: bool,
}

/// What a field gave: its value, the answer as the review shows it, and the options chosen in a choice.
pub(crate) struct Answer {
    pub(crate) value: Value,
    pub(crate) shown: String,
    pub(crate) chosen: Vec<usize>,
}

impl Answer {
    fn new(value: Value, shown: impl Into<String>) -> Self {
        Self {
            value,
            shown: shown.into(),
            chosen: Vec::new(),
        }
    }
}

/// How asking a field ended: with a new answer, or, when changing one from the review, with the old one kept.
pub(crate) enum Outcome {
    Answered(Answer),
    Kept,
}

impl Asking<'_> {
    pub(crate) fn ask(&self, console: &mut impl Console) -> Result<Outcome, Error> {
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
                let current = self.current().map(|(_, _, chosen)| chosen);
                let choice = Choice {
                    prompt: &self.prompt,
                    options,
                    mode: *mode,
                    defaults: current.unwrap_or(defaults),
                    messages: self.messages,
                    back: self.editing,
                };
                let Some(chosen) = console.choose(&choice)? else {
                    return if self.editing {
                        Ok(Outcome::Kept)
                    } else {
                        Err(self.unanswered(console))
                    };
                };
                let shown: Vec<&str> = chosen.iter().map(|&index| options[index].as_str()).collect();
                Ok(Outcome::Answered(Answer {
                    value: pick(&chosen),
                    shown: shown.join(", "),
                    chosen,
                }))
            }
        }
    }

    /// The answer the field has while it is changed from the review: its value, as shown, and the options chosen.
    fn current(&self) -> Option<(&Value, &str, &[usize])> {
        match &self.field.state {
            State::Answered { value, shown, chosen } if self.editing => Some((value, shown, chosen)),
            _ => None,
        }
    }

    /// The prompt shown while the answer is changed from the review: the answer it has, which Enter keeps, in place
    /// of the field's default.
    fn editing_prompt(&self, kept: &str) -> Prompt {
        let mut prompt = self.prompt.clone();
        if self.editing {
            prompt.default = Some(if kept.is_empty() {
                self.messages.empty_answer.clone()
            } else {
                kept.to_owned()
            });
        }
        prompt
    }

    fn text(
        &self,
        console: &mut impl Console,
        validator: Option<&Validator>,
        rules: &[ContextRule],
        parse: &Parse,
        fallback: Option<&Fallback>,
        optional: bool,
    ) -> Result<Outcome, Error> {
        let prompt = self.editing_prompt(self.current().map_or("", |(_, shown, _)| shown));
        loop {
            let line = self.line(console, &prompt, fallback.is_some() || optional)?;
            let input = line.trim();
            if input.is_empty() && self.editing {
                return Ok(Outcome::Kept);
            }
            let outcome = if input.is_empty() {
                self.empty(parse, fallback, optional)
            } else {
                self.take(input, validator, rules, parse)
            };
            match outcome {
                Ok(answer) => return Ok(Outcome::Answered(answer)),
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
            return Ok(Answer::new((fallback.value)(), fallback.shown.clone()));
        }
        if optional && let Ok(value) = parse("") {
            return Ok(Answer::new(value, ""));
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
            Ok(value) => Ok(Answer::new(value, input)),
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
    ) -> Result<Outcome, Error> {
        let prompt = self.editing_prompt(&self.messages.hidden_answer);
        loop {
            let first = console.read_secret(&prompt)?.ok_or_else(|| self.unanswered(console))?;
            if first.is_empty() {
                if self.editing {
                    return Ok(Outcome::Kept);
                }
                console.report(&self.messages.required)?;
                continue;
            }
            if let Some(message) = self.refusal(validator, rules, &first) {
                console.report(message)?;
                continue;
            }
            if let Some(repeat) = repeat.filter(|_| console.is_interactive()) {
                let mut again = Prompt::new(&self.field.name, repeat);
                again.step = self.prompt.step;
                again.marker.clone_from(&self.prompt.marker);
                let second = console.read_secret(&again)?.ok_or_else(|| self.unanswered(console))?;
                if second != first {
                    console.report(&self.messages.passwords_differ)?;
                    continue;
                }
            }
            return Ok(Outcome::Answered(Answer::new(
                Box::new(first),
                self.messages.hidden_answer.clone(),
            )));
        }
    }

    fn confirm(&self, console: &mut impl Console, default: Option<bool>) -> Result<Outcome, Error> {
        let current = self
            .current()
            .and_then(|(value, _, _)| value.downcast_ref::<bool>().copied());
        let default = current.or(default);
        let mut prompt = self.prompt.clone();
        let choices = format!("{}/{}", initial(&self.messages.yes), initial(&self.messages.no));
        prompt.hint = Some(match &prompt.hint {
            Some(hint) => format!("{choices}, {hint}"),
            None => choices,
        });
        prompt.default = default.map(|answer| self.word(answer).to_owned());
        loop {
            let line = self.line(console, &prompt, default.is_some())?;
            match self.yes_or_no(&line, default) {
                Some(answer) if current == Some(answer) && line.trim().is_empty() => return Ok(Outcome::Kept),
                Some(answer) => {
                    return Ok(Outcome::Answered(Answer::new(Box::new(answer), self.word(answer))));
                }
                None => console.report(&self.messages.answer_yes_or_no)?,
            }
        }
    }

    /// The form's word for `answer`: yes or no.
    fn word(&self, answer: bool) -> &str {
        if answer { &self.messages.yes } else { &self.messages.no }
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
