//! Asks a few questions in the terminal and prints the answers: `cargo run --example demo`.

use form_builder::validate::{self, Validator};
use form_builder::{Confirm, Error, FormBuilder, MultiSelect, Password, Select, Text};

fn main() -> Result<(), Error> {
    let countries: Vec<(&str, &str)> = [
        "Austria",
        "Belgium",
        "Croatia",
        "Czechia",
        "Denmark",
        "Estonia",
        "Finland",
        "France",
        "Germany",
        "Greece",
        "Hungary",
        "Ireland",
        "Italy",
        "Latvia",
        "Lithuania",
        "Netherlands",
        "Poland",
        "Portugal",
        "Slovakia",
        "Slovenia",
        "Spain",
        "Sweden",
    ]
    .iter()
    .map(|&country| (country, country))
    .collect();

    let mut form = FormBuilder::new()
        .field(
            Text::<String>::new("name", "Name:")
                .validator(Validator::new().rule(validate::name, "Name cannot contain digits")),
        )
        .field(
            Text::<String>::new("email", "Email:")
                .validator(Validator::new().rule(validate::email, "That is not an email address")),
        )
        .field(Text::<u32>::new("age", "Age:").default(30))
        .field(Text::<f64>::optional("height", "Height:").hint("in metres"))
        .field(Select::new("country", "Country:", countries).default(&"Poland"))
        .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro"), (3, "Team")]).default(&1))
        .field(
            MultiSelect::new(
                "hobbies",
                "Hobbies:",
                [("reading", "Reading"), ("sports", "Sports"), ("music", "Music")],
            )
            .defaults(&["music"])
            .limit(2)
            .hint("two at most"),
        )
        .field(Confirm::new("company", "Signing up for a company?").default(false))
        .field(Text::<String>::new("vat", "VAT ID:").when(|answers| answers.is("company", &true)))
        .field(
            Password::new("password", "Password:")
                .validator(Validator::new().rule(validate::min_length(8), "At least 8 characters"))
                .repeat("Repeat the password:"),
        )
        .build();

    form.fill()?;

    let name: String = form.get("name")?;
    let age: u32 = form.get("age")?;
    let height: Option<f64> = form.get("height")?;
    let country: &str = form.get("country")?;
    let plan: u8 = form.get("plan")?;
    let hobbies: Vec<&str> = form.get("hobbies")?;
    let vat = form.get::<String>("vat").ok();
    println!("{name}, {age}, height {height:?}, {country}, plan {plan}, hobbies {hobbies:?}, VAT {vat:?}");
    Ok(())
}
