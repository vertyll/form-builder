# Form Builder

Forms for the terminal: text fields parsed into any type, optional fields, and choices of one or several options
moved through with the arrow keys.

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

`cargo run --example demo` runs a longer form.

## Documentation

- [Contents](CONTENTS.md) – every document of the library, and what it covers.
- [Features](docs/features.md) – every kind of field, defaults, choices, validation, languages and errors.
- [Standards](STANDARDS.md) – the specifications the checks and the terminal follow.
- [API documentation](https://docs.rs/form_builder) – every type and method, on docs.rs.

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
