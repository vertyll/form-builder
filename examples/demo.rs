//! Asks a few questions in the terminal and prints the answers: `cargo run --example demo`.

use form_builder::validate::{self, Validator};
use form_builder::{Error, FormBuilder};

fn main() -> Result<(), Error> {
    let mut form = FormBuilder::new()
        .add_field::<String>(
            "name",
            "Name:",
            Some(
                Validator::new()
                    .rule(validate::not_empty, "Name cannot be empty")
                    .rule(validate::name, "Name cannot contain digits"),
            ),
        )
        .add_field::<String>(
            "email",
            "Email:",
            Some(Validator::new().rule(validate::email, "That is not an email address")),
        )
        .add_field::<u32>("age", "Age:", None)
        .add_optional_field::<f64>("height", "Height in metres (optional):", None)
        .add_select("plan", "Plan:", [(1u8, "Free"), (2, "Pro"), (3, "Team")])
        .add_multiselect(
            "hobbies",
            "Hobbies, two at most:",
            [("reading", "Reading"), ("sports", "Sports"), ("music", "Music")],
            Some(2),
        )
        .build();

    form.fill()?;

    let name: String = form.get("name")?;
    let email: String = form.get("email")?;
    let age: u32 = form.get("age")?;
    let height: Option<f64> = form.get("height")?;
    let plan: u8 = form.get("plan")?;
    let hobbies: Vec<&str> = form.get("hobbies")?;
    println!("{name} <{email}>, {age}, height {height:?}, plan {plan}, hobbies {hobbies:?}");
    Ok(())
}
