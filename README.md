# Form Builder

Forms for the terminal: text fields parsed into any type, optional fields, and choices of one or several options
moved through with the arrow keys.

## Features

- **Typed fields**: a text field is parsed into any type that implements `FromStr`, and read back as that type.
- **Defaults**: an empty answer takes the field's default, shown in grey after the prompt: `Port * [8080]:`. A
  select starts on its default option, marked `(default)`.
- **Optional fields**: an empty answer gives `None`.
- **Select and multiselect**: the user moves with the arrow keys or `j`/`k`, chooses with Space and confirms with
  Enter; Esc or Ctrl+C cancels. A list taller than the terminal scrolls, showing how many options are above and
  below. A multiselect may limit how many options are chosen.
- **Yes-or-no questions** answered `y` or `n`, and **passwords** typed unseen, asked twice when they must be repeated.
- **Conditional fields**: a field asked only when an earlier answer calls for it.
- **Validation**: a `Validator` checks a text field's input before it is parsed, and the field asks again with the
  message of the first check that fails. The checks in `validate` cover names, emails, dates, URLs and more, any
  closure can be one, and a rule can compare the input with earlier answers.
- **Required marks**: the prompt of every required field carries `*`, as `Name *:`; `required_marker("(required)")`
  uses a marker of your own, and `no_required_marker()` leaves it out.
- **Any language**: the texts the form shows of its own, such as `This field is required`, are in `Messages`, and
  can be replaced.
- **Errors you can match**: every failure is an `Error` variant, such as the input ending before a field was filled.
- **Linux, macOS and Windows**, through [crossterm](https://github.com/crossterm-rs/crossterm).

## Installation

```bash
cargo add form_builder
```

## Usage

Each kind of field has its own builder, added to the form with `field`:

```rust
use form_builder::validate::{self, Validator};
use form_builder::{Confirm, Error, FormBuilder, MultiSelect, Password, Select, Text};

fn main() -> Result<(), Error> {
    let mut form = FormBuilder::new()
        .field(Text::<String>::new("name", "Name:").validator(
            Validator::new().rule(validate::name, "Name cannot contain digits"),
        ))
        .field(Text::<u16>::new("port", "Port:").default(8080).hint("1-65535"))
        .field(Text::<f64>::optional("height", "Height in metres:"))
        .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&1))
        .field(MultiSelect::new("tags", "Tags:", [("rust", "Rust"), ("go", "Go")]).limit(2))
        .field(Confirm::new("company", "Signing up for a company?").default(false))
        .field(Text::<String>::new("vat", "VAT ID:").when(|answers| answers.is("company", &true)))
        .field(Password::new("password", "Password:").repeat("Repeat the password:"))
        .build();

    form.fill()?;

    let name: String = form.get("name")?;
    let port: u16 = form.get("port")?;
    let height: Option<f64> = form.get("height")?;
    let tags: Vec<&str> = form.get("tags")?;
    println!("{name}, {port}, {height:?}, {tags:?}");
    Ok(())
}
```

For the plain cases, `add_field`, `add_optional_field`, `add_select` and `add_multiselect` add a field in one call.
`cargo run --example demo` runs a longer form. `Form::fill_with` fills a form through any `Console`, such as one that
replays answers in a test.

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
