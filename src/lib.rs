//! Forms for the terminal: text parsed into any type, optional fields, defaults, yes-or-no questions, passwords, and
//! choices of one or several options moved through with the arrow keys.
//!
//! A [`FormBuilder`] collects the fields, [`Form::fill`] asks them in order, and [`Form::get`] reads each value by
//! name, as the type the field holds. Each kind of field has its own builder: [`Text`], [`Select`],
//! [`MultiSelect`], [`Confirm`] and [`Password`]; [`FormBuilder::add_field`] and its siblings are shorthands for the
//! plain cases.
//!
//! ```no_run
//! use form_builder::validate::{self, Validator};
//! use form_builder::{Confirm, Error, FormBuilder, MultiSelect, Password, Select, Text};
//!
//! fn main() -> Result<(), Error> {
//!     let mut form = FormBuilder::new()
//!         .field(Text::<String>::new("name", "Name:").validator(
//!             Validator::new().rule(validate::name, "Name cannot contain digits"),
//!         ))
//!         .field(Text::<u16>::new("port", "Port:").default(8080).hint("1-65535"))
//!         .field(Text::<f64>::optional("height", "Height in metres:"))
//!         .field(Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&1))
//!         .field(MultiSelect::new("tags", "Tags:", [("rust", "Rust"), ("go", "Go")]).limit(2))
//!         .field(Confirm::new("company", "Signing up for a company?").default(false))
//!         .field(Text::<String>::new("vat", "VAT ID:").when(|answers| answers.is("company", &true)))
//!         .field(Password::new("password", "Password:").repeat("Repeat the password:"))
//!         .build();
//!
//!     form.fill()?;
//!
//!     let port: u16 = form.get("port")?;
//!     let height: Option<f64> = form.get("height")?;
//!     let tags: Vec<&str> = form.get("tags")?;
//!     println!("{port}, {height:?}, {tags:?}");
//!     Ok(())
//! }
//! ```
//!
//! Every prompt starts with its step, and the prompt of every required field is marked with `*`; the step, a hint
//! and a default are shown in grey: `[2/8] Port * (1-65535) [8080]:`. [`FormBuilder::no_steps`] and
//! [`FormBuilder::no_required_marker`] leave them out, and [`FormBuilder::messages`] translates the texts the form
//! shows of its own. [`FormBuilder::review`] lists the answers at the end, to change any.
//!
//! [`Form::fill_with`] fills a form through any [`Console`]: a [`Preset`] answers it from the environment or the
//! command line, where nobody can, and a console that replays answers tests it. With the `derive` feature,
//! `#[derive(Form)]` asks for a struct field by field, through [`FromForm`].

mod console;
mod error;
mod fields;
mod form;
mod from_form;
mod preset;
mod prompt;
mod selection;
pub mod validate;

pub use console::{Choice, Console, Terminal};
pub use error::Error;
pub use fields::{Confirm, Field, MultiSelect, Password, Select, Text};
pub use form::{Answers, Form, FormBuilder};
#[cfg(feature = "derive")]
pub use form_builder_derive::Form;
pub use from_form::FromForm;
pub use preset::Preset;
pub use prompt::{Messages, Prompt};
pub use selection::Mode;
pub use validate::Validator;
