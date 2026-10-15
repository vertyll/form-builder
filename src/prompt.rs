//! What a field shows when it asks: its prompt, with the marks around it, and the built-in texts of a form.

use std::fmt;

/// A field's prompt, split into the parts a console may show differently: the step, the text, the mark of a required
/// field, a hint, and the answer an empty input gives.
///
/// As text it reads `[3/8] Port * (1-65535) [8080]:`; a [`Terminal`](crate::Terminal) shows the step, the hint and
/// the default in grey.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Prompt {
    /// The name of the field asked, which a [`Preset`](crate::Preset) finds its answer by.
    pub name: String,
    /// Which field of how many this is, counted from 1.
    pub step: Option<(usize, usize)>,
    /// The prompt as the field was given it, without its closing colon.
    pub label: String,
    /// The mark of a required field, such as `*`.
    pub marker: Option<String>,
    /// A short note on what to enter.
    pub hint: Option<String>,
    /// What an empty answer gives, as shown: `8080`, or `no` for a yes-or-no question.
    pub default: Option<String>,
    /// Whether the prompt ended with a colon, which goes back after the marks.
    pub colon: bool,
}

impl Prompt {
    pub(crate) fn new(name: &str, text: &str) -> Self {
        let text = text.trim_end();
        let (label, colon) = match text.strip_suffix(':') {
            Some(label) => (label.trim_end(), true),
            None => (text, false),
        };
        Self {
            name: name.to_owned(),
            step: None,
            label: label.to_owned(),
            marker: None,
            hint: None,
            default: None,
            colon,
        }
    }

    /// The step as shown: `[3/8]`.
    #[must_use]
    pub fn shown_step(&self) -> Option<String> {
        self.step.map(|(step, steps)| format!("[{step}/{steps}]"))
    }

    /// The text before the hint and the default: the label and the mark.
    #[must_use]
    pub fn lead(&self) -> String {
        match &self.marker {
            Some(marker) => format!("{} {marker}", self.label),
            None => self.label.clone(),
        }
    }

    /// The hint as shown: `(1-65535)`.
    #[must_use]
    pub fn shown_hint(&self) -> Option<String> {
        self.hint.as_ref().map(|hint| format!("({hint})"))
    }

    /// The default as shown: `[8080]`.
    #[must_use]
    pub fn shown_default(&self) -> Option<String> {
        self.default.as_ref().map(|default| format!("[{default}]"))
    }

    /// The colon the prompt ends with, if it had one.
    #[must_use]
    pub fn end(&self) -> &'static str {
        if self.colon { ":" } else { "" }
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(step) = self.shown_step() {
            write!(f, "{step} ")?;
        }
        f.write_str(&self.lead())?;
        for part in [self.shown_hint(), self.shown_default()].into_iter().flatten() {
            write!(f, " {part}")?;
        }
        f.write_str(self.end())
    }
}

/// The texts a form shows of its own, in English unless replaced: every field is free to be in another language,
/// and these follow it.
///
/// ```
/// use form_builder::{FormBuilder, Messages};
///
/// let form = FormBuilder::new().messages(Messages {
///     invalid_input: "Nieprawidłowa wartość".to_owned(),
///     ..Messages::default()
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messages {
    /// The keys that move the cursor of a choice, shown under its prompt.
    pub key_move: String,
    /// The key that chooses the option of a select.
    pub key_choose: String,
    /// The key that marks an option of a multiselect.
    pub key_mark: String,
    /// The key that confirms the options marked in a multiselect.
    pub key_confirm: String,
    /// The key that goes back to the defaults of a choice that has them.
    pub key_default: String,
    /// The key that cancels the form.
    pub key_cancel: String,
    /// The key that goes back to the review and keeps the answer being changed.
    pub key_back: String,
    /// What separates the keys in the line of controls.
    pub controls_separator: String,
    /// Shown when a required field gets an empty answer.
    pub required: String,
    /// Shown before why an input could not be read as the field's type.
    pub invalid_input: String,
    /// Shown when a yes-or-no question gets another answer.
    pub answer_yes_or_no: String,
    /// The answer that means yes; any answer starting with its first letter does.
    pub yes: String,
    /// The answer that means no; any answer starting with its first letter does.
    pub no: String,
    /// Shown when a repeated password differs from the first.
    pub passwords_differ: String,
    /// Shown after the prompt of a choice the user cancelled.
    pub cancelled: String,
    /// Shown after the option a choice starts on.
    pub default_option: String,
    /// Shown above a list cut to fit the terminal, with how many options are above.
    pub more_above: String,
    /// Shown below a list cut to fit the terminal, with how many options are below.
    pub more_below: String,
    /// The prompt of the review of the answers, before the form is done.
    pub review: String,
    /// The option of the review that accepts the answers.
    pub submit: String,
    /// How the review shows an optional field left empty.
    pub empty_answer: String,
    /// How the review shows a password.
    pub hidden_answer: String,
}

impl Default for Messages {
    fn default() -> Self {
        Self {
            key_move: "↑↓ move".to_owned(),
            key_choose: "Enter choose".to_owned(),
            key_mark: "Space mark".to_owned(),
            key_confirm: "Enter confirm".to_owned(),
            key_default: "d default".to_owned(),
            key_cancel: "Esc cancel".to_owned(),
            key_back: "Esc back".to_owned(),
            controls_separator: " · ".to_owned(),
            required: "This field is required".to_owned(),
            invalid_input: "Invalid input".to_owned(),
            answer_yes_or_no: "Please answer yes or no".to_owned(),
            yes: "yes".to_owned(),
            no: "no".to_owned(),
            passwords_differ: "The passwords do not match".to_owned(),
            cancelled: "cancelled".to_owned(),
            default_option: "default".to_owned(),
            more_above: "more above".to_owned(),
            more_below: "more below".to_owned(),
            review: "Check your answers, or choose one to change:".to_owned(),
            submit: "Submit".to_owned(),
            empty_answer: "-".to_owned(),
            hidden_answer: "********".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Prompt;

    #[test]
    fn a_prompt_reads_with_its_marks_before_the_colon() {
        let mut prompt = Prompt::new("port", "Port:");
        assert_eq!(prompt.to_string(), "Port:");
        prompt.marker = Some("*".to_owned());
        prompt.hint = Some("1-65535".to_owned());
        prompt.default = Some("8080".to_owned());
        assert_eq!(prompt.to_string(), "Port * (1-65535) [8080]:");
        prompt.step = Some((3, 8));
        assert_eq!(prompt.to_string(), "[3/8] Port * (1-65535) [8080]:");
    }

    #[test]
    fn a_prompt_without_a_colon_gets_none() {
        let mut prompt = Prompt::new("name", "Your name ");
        prompt.marker = Some("(required)".to_owned());
        assert_eq!(prompt.to_string(), "Your name (required)");
    }
}
