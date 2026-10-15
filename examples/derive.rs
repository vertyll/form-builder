//! A struct asked for field by field: `cargo run --example derive --features derive`.

use form_builder::{Error, Form, FromForm};

#[derive(Form, Debug)]
struct SignUp {
    first_name: String,
    #[form(prompt = "Port:", default = 8080, hint = "1024-65535")]
    port: u16,
    #[form(password, repeat = "Repeat the password:")]
    password: String,
    newsletter: bool,
    company: Option<String>,
}

fn main() -> Result<(), Error> {
    let mut form = SignUp::form().review().build();
    form.fill()?;
    let sign_up = SignUp::from_form(&form)?;
    println!(
        "{} on port {}, a password of {} characters, newsletter {}, company {:?}",
        sign_up.first_name,
        sign_up.port,
        sign_up.password.chars().count(),
        sign_up.newsletter,
        sign_up.company
    );
    Ok(())
}
