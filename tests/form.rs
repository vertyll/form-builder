//! Forms filled through a console that replays scripted answers.

use std::collections::VecDeque;

use form_builder::validate::{self, Validator};
use form_builder::{Console, Error, FormBuilder, Mode};

#[derive(Default)]
struct Scripted {
    lines: VecDeque<&'static str>,
    choices: VecDeque<Option<Vec<usize>>>,
    reports: Vec<String>,
    prompts: Vec<String>,
}

impl Scripted {
    fn new(lines: &[&'static str], choices: Vec<Option<Vec<usize>>>) -> Self {
        Self {
            lines: lines.iter().copied().collect(),
            choices: choices.into(),
            ..Self::default()
        }
    }
}

impl Console for Scripted {
    fn read_line(&mut self, prompt: &str) -> Result<Option<String>, Error> {
        self.prompts.push(prompt.to_owned());
        Ok(self.lines.pop_front().map(str::to_owned))
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        self.reports.push(message.to_owned());
        Ok(())
    }

    fn choose(&mut self, prompt: &str, _: &[String], _: Mode) -> Result<Option<Vec<usize>>, Error> {
        self.prompts.push(prompt.to_owned());
        Ok(self.choices.pop_front().flatten())
    }
}

#[test]
fn every_kind_of_field_is_filled_and_read_as_its_type() {
    let mut form = FormBuilder::new()
        .add_field::<String>("name", "Name:", None)
        .add_field::<u32>("age", "Age:", None)
        .add_optional_field::<f64>("height", "Height:", None)
        .add_optional_field::<f64>("width", "Width:", None)
        .add_select("plan", "Plan:", [(1u8, "Free"), (2, "Pro")])
        .add_multiselect(
            "hobbies",
            "Hobbies:",
            [("a, b", "Commas"), ("\"q\"", "Quotes"), ("x", "X")],
            None,
        )
        .build();
    let mut console = Scripted::new(&["  Ann  ", "42", "1.8", ""], vec![Some(vec![1]), Some(vec![0, 1])]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<String>("name").unwrap(), "Ann");
    assert_eq!(form.get::<u32>("age").unwrap(), 42);
    assert_eq!(form.get::<Option<f64>>("height").unwrap(), Some(1.8));
    assert_eq!(form.get::<Option<f64>>("width").unwrap(), None);
    assert_eq!(form.get::<u8>("plan").unwrap(), 2);
    assert_eq!(form.get::<Vec<&str>>("hobbies").unwrap(), ["a, b", "\"q\""]);
    assert_eq!(
        console.prompts,
        ["Name:", "Age:", "Height:", "Width:", "Plan:", "Hobbies:"]
    );
}

#[test]
fn refused_input_is_reported_and_asked_again() {
    let validator = Validator::new()
        .rule(validate::not_empty, "Required")
        .rule(validate::name, "No digits");
    let mut form = FormBuilder::new()
        .add_field::<String>("name", "Name:", Some(validator))
        .add_field::<u32>("age", "Age:", None)
        .build();
    let mut console = Scripted::new(&["", "R2D2", "Ann", "old", "42"], vec![]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<String>("name").unwrap(), "Ann");
    assert_eq!(console.reports[..2], ["Required", "No digits"]);
    assert!(
        console.reports[2].starts_with("Invalid input: "),
        "{:?}",
        console.reports
    );
}

#[test]
fn an_empty_optional_field_skips_its_validator() {
    let validator = Validator::new().rule(validate::not_empty, "Required");
    let mut form = FormBuilder::new()
        .add_optional_field::<u32>("age", "Age:", Some(validator))
        .build();
    form.fill_with(&mut Scripted::new(&[""], vec![])).unwrap();
    assert_eq!(form.get::<Option<u32>>("age").unwrap(), None);
}

#[test]
fn the_end_of_the_input_stops_the_form_instead_of_asking_forever() {
    let mut form = FormBuilder::new().add_field::<u32>("age", "Age:", None).build();
    let error = form.fill_with(&mut Scripted::new(&["old"], vec![])).unwrap_err();
    assert!(matches!(error, Error::EndOfInput(ref name) if name == "age"), "{error}");
}

#[test]
fn a_choice_without_options_or_cancelled_is_an_error() {
    let mut empty = FormBuilder::new().add_select::<u8, &str>("plan", "Plan:", []).build();
    let error = empty.fill_with(&mut Scripted::default()).unwrap_err();
    assert!(matches!(error, Error::NoOptions(_)), "{error}");
    let mut form = FormBuilder::new().add_select("plan", "Plan:", [(1u8, "Free")]).build();
    let error = form.fill_with(&mut Scripted::new(&[], vec![None])).unwrap_err();
    assert!(matches!(error, Error::Cancelled(_)), "{error}");
}

#[test]
fn reading_a_value_names_what_is_wrong() {
    let mut form = FormBuilder::new().add_field::<u32>("age", "Age:", None).build();
    assert!(matches!(form.get::<u32>("age"), Err(Error::NotFilled(_))));
    form.fill_with(&mut Scripted::new(&["42"], vec![])).unwrap();
    assert!(matches!(form.get::<u32>("height"), Err(Error::NotFound(_))));
    let error = form.get::<String>("age").unwrap_err();
    let message = error.to_string();
    assert!(
        message.starts_with("field `age` does not hold a `") && message.contains("String"),
        "{message}"
    );
}
