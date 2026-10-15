//! A form answered without asking, as in CI: `APP_NAME=Ann cargo run --example preset -- port=9000 plan=pro`.
//!
//! The answers come from the environment variables that start with `APP_` and from the arguments, the arguments
//! winning. A field left without an answer takes its default; one with neither ends the form with an error that
//! names it, as does an answer the form refuses.

use form_builder::{Error, FormBuilder, Preset, Select, Text};

fn main() -> Result<(), Error> {
    let mut form = FormBuilder::new()
        .field(Text::<String>::new("name", "Name:"))
        .field(
            Text::<u16>::new("port", "Port:")
                .default(8080)
                .check(|port| *port >= 1024, "Use a port above 1023"),
        )
        .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&1))
        .build();

    let mut answers = Preset::from_env("APP_").merge(Preset::from_args(std::env::args().skip(1)));
    form.fill_with(&mut answers)?;

    let name: String = form.get("name")?;
    let port: u16 = form.get("port")?;
    let plan: u8 = form.get("plan")?;
    println!("{name} on port {port}, plan {plan}");
    Ok(())
}
