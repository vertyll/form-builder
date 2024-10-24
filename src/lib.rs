//! A library for building forms with various fields and validation.
//!
//! This library provides a flexible way to create forms with different types of fields, including text fields, select fields, and multiselect fields. It also supports validation for these fields.
//!
//! # Example
//!
//! ```rust
//! use form_builder::{FormBuilder, Optional, ValidationMethods, Validator};
//!
//! fn validate_custom(value: &str) -> bool {
//!     value.len() > 5
//! }
//!
//! fn main() -> Result<(), String> {
//!     let mut form = FormBuilder::new()
//!         .add_field::<String>(
//!             "name",
//!             "Enter name:",
//!             Some(Validator::new(vec![
//!                 (ValidationMethods::not_empty, Some("Name cannot be empty")),
//!                 (
//!                     ValidationMethods::validate_name,
//!                     Some("Name cannot contain numbers"),
//!                 ),
//!             ])),
//!         )
//!         .add_field::<String>(
//!             "email",
//!             "Enter email:",
//!             Some(Validator::new(vec![
//!                 (ValidationMethods::not_empty, Some("Email cannot be empty")),
//!                 (
//!                     ValidationMethods::validate_email,
//!                     Some("Invalid email format"),
//!                 ),
//!             ])),
//!         )
//!         .add_field::<u32>(
//!             "age",
//!             "Enter age:",
//!             Some(Validator::new(vec![(
//!                 ValidationMethods::not_empty,
//!                 Some("Age cannot be empty"),
//!             )])),
//!         )
//!         .add_field::<String>(
//!             "custom",
//!             "Enter custom value:",
//!             Some(Validator::new(vec![
//!                 (
//!                     ValidationMethods::not_empty,
//!                     Some("Custom value cannot be empty"),
//!                 ),
//!                 (
//!                     validate_custom,
//!                     Some("Custom value must be longer than 5 characters"),
//!                 ),
//!             ])),
//!         )
//!         .add_field::<f64>(
//!             "height",
//!             "Enter height:",
//!             Some(Validator::new(vec![(
//!                 ValidationMethods::not_empty,
//!                 Some("Height cannot be empty"),
//!             )])),
//!         )
//!         .add_field::<bool>(
//!             "is_student",
//!             "Are you a student (true/false):",
//!             Some(Validator::new(vec![(
//!                 ValidationMethods::not_empty,
//!                 Some("This field cannot be empty"),
//!             )])),
//!         )
//!         .add_field::<char>(
//!             "initial",
//!             "Enter your initial:",
//!             Some(Validator::new(vec![(
//!                 ValidationMethods::not_empty,
//!                 Some("Initial cannot be empty"),
//!             )])),
//!         )
//!         .add_field::<Optional<u32>>("width", "Enter width (optional):", None)
//!         .add_select(
//!             "gender",
//!             "Select your gender:",
//!             vec![
//!                 (1u32, "Male"),
//!                 (2u32, "Female"),
//!                 (3u32, "Other"),
//!             ],
//!         )
//!         .add_multiselect(
//!             "hobbies",
//!             "Select your hobbies:",
//!             vec![
//!                 ("reading".to_string(), "Reading"),
//!                 ("sports".to_string(), "Sports"),
//!                 ("music".to_string(), "Music"),
//!             ],
//!             Some(2),
//!         )
//!         .build();
//!
//!     form.fill()?;
//!
//!     let name: String = form.get_value("name")?;
//!     let email: String = form.get_value("email")?;
//!     let age: u32 = form.get_value("age")?;
//!     let custom: String = form.get_value("custom")?;
//!     let height: f64 = form.get_value("height")?;
//!     let is_student: bool = form.get_value("is_student")?;
//!     let initial: char = form.get_value("initial")?;
//!     let width: Optional<u32> = form.get_value("width")?;
//!     let gender: u32 = form.get_value("gender")?;
//!     let hobbies: Vec<String> = form.get_value_vec("hobbies")?;
//!
//!     let width = process_width(width);
//!
//!     println!(
//!         "Name: {:?}, Email: {:?}, Age: {:?}, Custom: {:?}, Height: {:?}, Is Student: {:?}, Initial: {:?}, Width: {:?}, Gender: {:?}, Hobbies: {:?}",
//!         name, email, age, custom, height, is_student, initial, width, gender, hobbies
//!     );
//!
//!     Ok(())
//! }
//!
//! fn process_width(width: Optional<u32>) -> u32 {
//!     match width {
//!         Optional::Some(value) => value + 20,
//!         Optional::None => 0,
//!     }
//! }
//! ```

/// Module containing definitions for form fields.
pub mod field;

/// Module containing definitions for the form.
pub mod form;

/// Module containing the form builder.
pub mod form_builder;

/// Module containing functions for reading input.
pub mod input;

/// Module containing definitions for multiselect fields.
pub mod multiselect_field;

/// Module containing definitions for optional values.
pub mod optional;

/// Module containing definitions for select fields.
pub mod select_field;

/// Module containing validation methods.
pub mod validation;

pub use form_builder::FormBuilder;
pub use optional::Optional;
pub use validation::{ValidationMethods, Validator};
