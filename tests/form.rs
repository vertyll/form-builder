//! Forms filled through a console that replays scripted answers.

use std::collections::VecDeque;

use form_builder::validate::{self, Validator};
use form_builder::{
    Choice, Confirm, Console, Error, FormBuilder, Messages, MultiSelect, Password, Prompt, Select, Text,
};

#[derive(Default)]
struct Scripted {
    lines: VecDeque<&'static str>,
    choices: VecDeque<Option<Vec<usize>>>,
    reports: Vec<String>,
    prompts: Vec<String>,
    defaults: Vec<Vec<usize>>,
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
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        self.prompts.push(prompt.to_string());
        Ok(self.lines.pop_front().map(str::to_owned))
    }

    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        self.read_line(prompt)
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        self.reports.push(message.to_owned());
        Ok(())
    }

    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error> {
        self.prompts.push(choice.prompt.to_string());
        self.defaults.push(choice.defaults.to_vec());
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
        .field(Confirm::new("newsletter", "Newsletter?"))
        .field(Password::new("password", "Password:"))
        .build();
    let mut console = Scripted::new(
        &["  Ann  ", "42", "1.8", "", "y", "secret"],
        vec![Some(vec![1]), Some(vec![0, 1])],
    );
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<String>("name").unwrap(), "Ann");
    assert_eq!(form.get::<u32>("age").unwrap(), 42);
    assert_eq!(form.get::<Option<f64>>("height").unwrap(), Some(1.8));
    assert_eq!(form.get::<Option<f64>>("width").unwrap(), None);
    assert_eq!(form.get::<u8>("plan").unwrap(), 2);
    assert_eq!(form.get::<Vec<&str>>("hobbies").unwrap(), ["a, b", "\"q\""]);
    assert!(form.get::<bool>("newsletter").unwrap());
    assert_eq!(form.get::<String>("password").unwrap(), "secret");
    assert_eq!(
        console.prompts,
        [
            "Name *:",
            "Age *:",
            "Height:",
            "Width:",
            "Plan *:",
            "Hobbies *:",
            "Newsletter? * [y/n]",
            "Password *:"
        ]
    );
}

#[test]
fn refused_input_is_reported_and_asked_again() {
    let validator = Validator::new().rule(validate::name, "No digits");
    let mut form = FormBuilder::new()
        .add_field::<String>("name", "Name:", Some(validator))
        .add_field::<u32>("age", "Age:", None)
        .build();
    let mut console = Scripted::new(&["", "R2D2", "Ann", "old", "42"], vec![]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<String>("name").unwrap(), "Ann");
    assert_eq!(console.reports[..2], ["This field is required", "No digits"]);
    assert!(
        console.reports[2].starts_with("Invalid input: "),
        "{:?}",
        console.reports
    );
}

#[test]
fn an_empty_answer_takes_the_default_shown_in_the_prompt() {
    let mut form = FormBuilder::new()
        .field(Text::<u16>::new("port", "Port:").default(8080).hint("1-65535"))
        .field(Text::<u16>::optional("backup", "Backup port:").default(8081))
        .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&2))
        .field(MultiSelect::new("tags", "Tags:", [("a", "A"), ("b", "B"), ("c", "C")]).defaults(&["c", "a"]))
        .field(Confirm::new("newsletter", "Newsletter?").default(true))
        .build();
    let mut console = Scripted::new(&["", "", ""], vec![Some(vec![1]), Some(vec![0, 2])]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<u16>("port").unwrap(), 8080);
    assert_eq!(form.get::<Option<u16>>("backup").unwrap(), Some(8081));
    assert!(form.get::<bool>("newsletter").unwrap());
    assert_eq!(console.prompts[0], "Port * (1-65535) [8080]:");
    assert_eq!(console.prompts[1], "Backup port [8081]:");
    assert_eq!(console.prompts[4], "Newsletter? * [Y/n]");
    assert_eq!(console.defaults, [vec![1], vec![0, 2]]);
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
fn a_yes_or_no_question_takes_words_in_the_form_s_language() {
    let messages = Messages {
        yes: "tak".to_owned(),
        no: "nie".to_owned(),
        answer_yes_or_no: "Odpowiedz tak albo nie".to_owned(),
        ..Messages::default()
    };
    let mut form = FormBuilder::new()
        .messages(messages)
        .field(Confirm::new("ok", "Zgoda?").default(false))
        .build();
    let mut console = Scripted::new(&["y", "TAK"], vec![]);
    form.fill_with(&mut console).unwrap();
    assert!(form.get::<bool>("ok").unwrap());
    assert_eq!(console.reports, ["Odpowiedz tak albo nie"]);
    assert_eq!(console.prompts[0], "Zgoda? * [t/N]");
}

#[test]
fn a_repeated_password_must_match() {
    let mut form = FormBuilder::new()
        .field(Password::new("password", "Password:").repeat("Repeat:"))
        .build();
    let mut console = Scripted::new(&["one", "two", "same", "same"], vec![]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<String>("password").unwrap(), "same");
    assert_eq!(console.reports, ["The passwords do not match"]);
    assert_eq!(
        console.prompts,
        ["Password *:", "Repeat *:", "Password *:", "Repeat *:"]
    );
}

#[test]
fn a_field_is_asked_only_when_its_condition_holds() {
    let build = || {
        FormBuilder::new()
            .field(Confirm::new("company", "Company?"))
            .field(Text::<String>::new("vat", "VAT ID:").when(|answers| answers.is("company", &true)))
            .build()
    };
    let mut private = build();
    private.fill_with(&mut Scripted::new(&["n"], vec![])).unwrap();
    assert!(matches!(private.get::<String>("vat"), Err(Error::Skipped(_))));
    let mut company = build();
    company.fill_with(&mut Scripted::new(&["y", "PL123"], vec![])).unwrap();
    assert_eq!(company.get::<String>("vat").unwrap(), "PL123");
}

#[test]
fn a_rule_sees_the_answers_before_it() {
    let mut form = FormBuilder::new()
        .add_field::<u32>("min", "Minimum:", None)
        .field(Text::<u32>::new("max", "Maximum:").rule_with(
            |answers, input| {
                input
                    .parse::<u32>()
                    .is_ok_and(|max| answers.get::<u32>("min").is_ok_and(|min| max >= min))
            },
            "The maximum cannot be below the minimum",
        ))
        .build();
    let mut console = Scripted::new(&["10", "5", "20"], vec![]);
    form.fill_with(&mut console).unwrap();
    assert_eq!(form.get::<u32>("max").unwrap(), 20);
    assert_eq!(console.reports, ["The maximum cannot be below the minimum"]);
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
    let message = form.get::<String>("age").unwrap_err().to_string();
    assert!(
        message.starts_with("field `age` does not hold a `") && message.contains("String"),
        "{message}"
    );
    assert!(form.answers().is_answered("age") && !form.answers().is_answered("height"));
}

#[test]
fn required_fields_are_marked_unless_asked_not_to() {
    let build = |builder: FormBuilder| {
        builder
            .add_field::<String>("name", "Name:", None)
            .add_optional_field::<u32>("age", "Age:", None)
            .add_select("plan", "Plan", [(1u8, "Free")])
            .build()
    };
    for (builder, want) in [
        (FormBuilder::new(), ["Name *:", "Age:", "Plan *"]),
        (FormBuilder::new().no_required_marker(), ["Name:", "Age:", "Plan"]),
        (
            FormBuilder::new().required_marker("(required)"),
            ["Name (required):", "Age:", "Plan (required)"],
        ),
    ] {
        let mut form = build(builder);
        let mut console = Scripted::new(&["Ann", ""], vec![Some(vec![0])]);
        form.fill_with(&mut console).unwrap();
        assert_eq!(console.prompts, want);
    }
}
