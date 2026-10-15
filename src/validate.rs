//! Checks for text input, and the [`Validator`] that applies them to a field.
//!
//! Each check takes the trimmed input and says whether it is acceptable. Checks that need a parameter, such as
//! [`min_length`], return the check itself.

use std::fmt;
use std::sync::LazyLock;

use regex::Regex;

/// A rule of a [`Validator`]: a check and the message shown when the input fails it.
struct Rule {
    check: Box<dyn Fn(&str) -> bool + Send + Sync>,
    message: String,
}

/// The checks a field's input must pass, in order; the first that fails gives its message, and the field asks again.
///
/// ```
/// use form_builder::validate::{self, Validator};
///
/// let validator = Validator::new()
///     .rule(validate::not_empty, "Name cannot be empty")
///     .rule(validate::min_length(2), "Name is too short");
/// assert_eq!(validator.check("A"), Err("Name is too short"));
/// assert_eq!(validator.check("Ann"), Ok(()));
/// ```
#[derive(Default)]
pub struct Validator {
    rules: Vec<Rule>,
}

impl Validator {
    /// A validator with no rules, which accepts any input.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rule: `check` must hold for the input, or `message` is shown.
    #[must_use]
    pub fn rule(mut self, check: impl Fn(&str) -> bool + Send + Sync + 'static, message: impl Into<String>) -> Self {
        self.rules.push(Rule {
            check: Box::new(check),
            message: message.into(),
        });
        self
    }

    /// Checks `input` against every rule, in order.
    ///
    /// # Errors
    ///
    /// Returns the message of the first rule the input fails.
    pub fn check(&self, input: &str) -> Result<(), &str> {
        match self.rules.iter().find(|rule| !(rule.check)(input)) {
            Some(rule) => Err(&rule.message),
            None => Ok(()),
        }
    }
}

impl fmt::Debug for Validator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<&str> = self.rules.iter().map(|rule| rule.message.as_str()).collect();
        f.debug_struct("Validator").field("rules", &messages).finish()
    }
}

fn pattern(source: &str) -> Regex {
    Regex::new(source).expect("the pattern is valid")
}

static EMAIL: LazyLock<Regex> = LazyLock::new(|| pattern(r"^[\w.-]+@[\w.-]+\.\w+$"));
static DATE: LazyLock<Regex> = LazyLock::new(|| pattern(r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$"));
static TIME: LazyLock<Regex> = LazyLock::new(|| pattern(r"^([01]\d|2[0-3]):[0-5]\d:[0-5]\d$"));
static URL: LazyLock<Regex> = LazyLock::new(|| pattern(r"^https?://[^\s/$.?#][^\s]*$"));
static PHONE_NUMBER: LazyLock<Regex> = LazyLock::new(|| pattern(r"^\+?[1-9]\d{1,14}$"));
static POSTAL_CODE: LazyLock<Regex> = LazyLock::new(|| pattern(r"^\d{5}(-\d{4})?$"));
static CREDIT_CARD: LazyLock<Regex> = LazyLock::new(|| pattern(r"^\d{4}-?\d{4}-?\d{4}-?\d{4}$"));
static UUID: LazyLock<Regex> =
    LazyLock::new(|| pattern(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"));

/// The input is not empty.
#[must_use]
pub fn not_empty(input: &str) -> bool {
    !input.is_empty()
}

/// The input holds no digit, as a person's name does not.
#[must_use]
pub fn name(input: &str) -> bool {
    !input.chars().any(char::is_numeric)
}

/// The input is shaped as an email address: `local@domain.tld`.
#[must_use]
pub fn email(input: &str) -> bool {
    EMAIL.is_match(input)
}

/// The input is at least `min` characters long.
pub fn min_length(min: usize) -> impl Fn(&str) -> bool + Send + Sync {
    move |input| input.chars().count() >= min
}

/// The input is at most `max` characters long.
pub fn max_length(max: usize) -> impl Fn(&str) -> bool + Send + Sync {
    move |input| input.chars().count() <= max
}

/// The input is letters only.
#[must_use]
pub fn alphabetic(input: &str) -> bool {
    input.chars().all(char::is_alphabetic)
}

/// The input is a whole number that fits an `i64`.
#[must_use]
pub fn integer(input: &str) -> bool {
    input.parse::<i64>().is_ok()
}

/// The input is a number, as an `f64` reads one.
#[must_use]
pub fn float(input: &str) -> bool {
    input.parse::<f64>().is_ok()
}

/// The input is a date written `YYYY-MM-DD`.
#[must_use]
pub fn date(input: &str) -> bool {
    DATE.is_match(input)
}

/// The input is a time of day written `HH:MM:SS`.
#[must_use]
pub fn time(input: &str) -> bool {
    TIME.is_match(input)
}

/// The input is an `http` or `https` URL.
#[must_use]
pub fn url(input: &str) -> bool {
    URL.is_match(input)
}

/// The input is a phone number in E.164 form: up to 15 digits, with an optional leading `+`.
#[must_use]
pub fn phone_number(input: &str) -> bool {
    PHONE_NUMBER.is_match(input)
}

/// The input is a US ZIP code: five digits, with an optional `-` and four more.
#[must_use]
pub fn postal_code(input: &str) -> bool {
    POSTAL_CODE.is_match(input)
}

/// The input is shaped as a card number: four groups of four digits, with or without `-` between them.
#[must_use]
pub fn credit_card(input: &str) -> bool {
    CREDIT_CARD.is_match(input)
}

/// The input is a UUID in its hyphenated form.
#[must_use]
pub fn uuid(input: &str) -> bool {
    UUID.is_match(input)
}

#[cfg(test)]
mod tests {
    use super::{
        Validator, alphabetic, credit_card, date, email, float, integer, max_length, min_length, name, not_empty,
        phone_number, postal_code, time, url, uuid,
    };

    type Case = (fn(&str) -> bool, &'static str, &'static str);

    #[test]
    fn checks_accept_and_refuse() {
        let cases: [Case; 13] = [
            (name, "John", "John123"),
            (email, "test@example.com", "test@.com"),
            (not_empty, "x", ""),
            (alphabetic, "hello", "hello123"),
            (integer, "-123", "123abc"),
            (float, "123.45", "123.45abc"),
            (date, "2023-10-01", "2023-10-32"),
            (time, "12:34:56", "25:34:56"),
            (url, "https://example.com", "htp://example.com"),
            (phone_number, "+1234567890", "123-456-7890"),
            (postal_code, "12345-6789", "1234"),
            (credit_card, "1234-5678-1234-5678", "1234-5678-1234-567"),
            (
                uuid,
                "123e4567-e89b-12d3-a456-426614174000",
                "123e4567-e89b-12d3-a456-42661417400",
            ),
        ];
        for (check, good, bad) in cases {
            assert!(check(good), "{good}");
            assert!(!check(bad), "{bad}");
        }
        assert!(!email("test@com"));
    }

    #[test]
    fn lengths_count_characters() {
        assert!(min_length(3)("żółw"));
        assert!(!min_length(5)("żółw"));
        assert!(max_length(4)("żółw"));
        assert!(!max_length(3)("żółw"));
    }

    #[test]
    fn a_validator_gives_the_first_failed_message() {
        let validator = Validator::new().rule(not_empty, "empty").rule(name, "digits");
        assert_eq!(validator.check(""), Err("empty"));
        assert_eq!(validator.check("R2D2"), Err("digits"));
        assert_eq!(validator.check("Ann"), Ok(()));
        assert_eq!(Validator::new().check("anything"), Ok(()));
    }

    #[test]
    fn a_rule_may_be_a_closure() {
        let validator = Validator::new().rule(|input: &str| input.ends_with('!'), "shout it");
        assert_eq!(validator.check("hi"), Err("shout it"));
        assert_eq!(validator.check("hi!"), Ok(()));
    }
}
