# Form Builder

Forms for the terminal: text fields parsed into any type, optional fields, and choices of one or several options
moved through with the arrow keys.

## Features

- **Typed fields**: a text field is parsed into any type that implements `FromStr`, and read back as that type.
- **Optional fields**: an empty answer gives `None`.
- **Select and multiselect**: the user moves with the arrow keys or `j`/`k`, chooses with Space and confirms with
  Enter; Esc or Ctrl+C cancels. A multiselect may limit how many options are chosen.
- **Validation**: a `Validator` checks a text field's input before it is parsed, and the field asks again with the
  message of the first check that fails. The checks in `validate` cover names, emails, dates, URLs and more, and any
  closure can be one.
- **Errors you can match**: every failure is an `Error` variant, such as the input ending before a field was filled.
- **Linux, macOS and Windows**, through [crossterm](https://github.com/crossterm-rs/crossterm).

## Installation

```bash
cargo add form_builder
```

## Usage

```rust
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
        .add_field::<u32>("age", "Age:", None)
        .add_optional_field::<f64>("height", "Height in metres (optional):", None)
        .add_select("plan", "Plan:", [(1u8, "Free"), (2, "Pro")])
        .add_multiselect("hobbies", "Hobbies:", [("reading", "Reading"), ("music", "Music")], Some(2))
        .build();

    form.fill()?;

    let name: String = form.get("name")?;
    let age: u32 = form.get("age")?;
    let height: Option<f64> = form.get("height")?;
    let plan: u8 = form.get("plan")?;
    let hobbies: Vec<&str> = form.get("hobbies")?;
    println!("{name}, {age}, {height:?}, plan {plan}, {hobbies:?}");
    Ok(())
}
```

`cargo run --example demo` runs a longer form. `Form::fill_with` fills a form through any `Console`, such as one
that replays answers in a test.

## Migrating from 1.x

Version 2.0 changes the API: errors, reading values, optional fields and validators. [Migrating from 1.x to
2.0](docs/migration.md) shows each change, before and after.

## Documentation

The API documentation, with every check of `validate`, is at [docs.rs](https://docs.rs/form_builder).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Dependency Licenses

`form_builder` depends on [crossterm](https://github.com/crossterm-rs/crossterm), under MIT, and on
[regex](https://github.com/rust-lang/regex) and [thiserror](https://github.com/dtolnay/thiserror), under MIT or
Apache-2.0, at your choice. Cargo fetches them with their own license files; a program that ships `form_builder`
compiled in ships them too, and includes their licenses with it.

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue on GitHub. If you would
like to contribute code, please fork the repository and submit a pull request.
