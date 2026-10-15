//! Forms for the terminal: text fields parsed into any type that implements [`FromStr`](std::str::FromStr),
//! optional fields, and choices of one or several options moved through with the arrow keys.
//!
//! A [`FormBuilder`] collects the fields, [`Form::fill`] asks them in order, and [`Form::get`] reads each value by
//! name, as the type the field holds. A [`Validator`] checks a text field's input before it is parsed; the checks in
//! [`validate`] cover the common cases, and any closure can be one.
//!
//! ```no_run
//! use form_builder::validate::{self, Validator};
//! use form_builder::{Error, FormBuilder};
//!
//! fn main() -> Result<(), Error> {
//!     let mut form = FormBuilder::new()
//!         .add_field::<String>(
//!             "name",
//!             "Name:",
//!             Some(Validator::new()
//!                 .rule(validate::not_empty, "Name cannot be empty")
//!                 .rule(validate::name, "Name cannot contain digits")),
//!         )
//!         .add_field::<u32>("age", "Age:", None)
//!         .add_optional_field::<f64>("height", "Height in metres (optional):", None)
//!         .add_select("plan", "Plan:", [(1u8, "Free"), (2, "Pro")])
//!         .add_multiselect("hobbies", "Hobbies:", [("reading", "Reading"), ("music", "Music")], Some(2))
//!         .build();
//!
//!     form.fill()?;
//!
//!     let name: String = form.get("name")?;
//!     let age: u32 = form.get("age")?;
//!     let height: Option<f64> = form.get("height")?;
//!     let plan: u8 = form.get("plan")?;
//!     let hobbies: Vec<&str> = form.get("hobbies")?;
//!     println!("{name}, {age}, {height:?}, plan {plan}, {hobbies:?}");
//!     Ok(())
//! }
//! ```
//!
//! The prompt of every required field is marked with `*`, which [`FormBuilder::required_marker`] changes and
//! [`FormBuilder::no_required_marker`] leaves out. [`Form::fill_with`] fills a form through any [`Console`], such as
//! one that replays answers in a test.

mod console;
mod error;
mod form;
mod selection;
pub mod validate;

pub use console::{Console, Terminal};
pub use error::Error;
pub use form::{Form, FormBuilder};
pub use selection::Mode;
pub use validate::Validator;
