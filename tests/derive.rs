//! A struct filled through `#[derive(Form)]`.

#![cfg(feature = "derive")]

use form_builder::{Choice, Console, Error, Form, FromForm, Preset, Prompt};

#[derive(Form, Debug, PartialEq)]
struct SignUp {
    first_name: String,
    #[form(prompt = "Port:", default = 8080, hint = "1024-65535")]
    port: u16,
    #[form(password, repeat = "Repeat the password:")]
    password: String,
    newsletter: bool,
    company: Option<String>,
    #[form(skip)]
    source: String,
}

#[test]
fn a_struct_is_filled_field_by_field() {
    let mut preset = Preset::new([
        ("first_name", "Ann"),
        ("port", "9000"),
        ("password", "secret"),
        ("newsletter", "no"),
    ]);
    let from_preset = SignUp::fill_with(&mut preset).unwrap();
    assert_eq!(from_preset.port, 9000);
    assert!(
        !from_preset.newsletter && from_preset.company.is_none(),
        "an optional field without an answer is None"
    );

    let sign_up = SignUp::fill_with(&mut Answers::default()).unwrap();
    assert_eq!(
        sign_up,
        SignUp {
            first_name: "Ann".to_owned(),
            port: 8080,
            password: "secret".to_owned(),
            newsletter: true,
            company: None,
            source: String::new(),
        }
    );
}

#[derive(Default)]
struct Answers {
    prompts: Vec<String>,
}

impl Console for Answers {
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        let mut shown = prompt.clone();
        shown.step = None;
        self.prompts.push(shown.to_string());
        Ok(Some(
            match prompt.name.as_str() {
                "first_name" => "Ann",
                "newsletter" => "y",
                _ => "",
            }
            .to_owned(),
        ))
    }

    fn read_secret(&mut self, _: &Prompt) -> Result<Option<String>, Error> {
        Ok(Some("secret".to_owned()))
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        panic!("refused: {message}")
    }

    fn choose(&mut self, _: &Choice<'_>) -> Result<Option<Vec<usize>>, Error> {
        Ok(None)
    }
}

#[test]
fn prompts_come_from_the_names_and_the_attributes() {
    let mut console = Answers::default();
    SignUp::fill_with(&mut console).unwrap();
    assert_eq!(
        console.prompts,
        [
            "First name *:",
            "Port * (1024-65535) [8080]:",
            "Newsletter * [y/n]:",
            "Company:"
        ]
    );
}
