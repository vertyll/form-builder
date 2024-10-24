use crate::field::Field;
use crate::field::FieldTrait;
use crate::form::Form;
use crate::multiselect_field::MultiselectField;
use crate::select_field::SelectField;
use crate::validation::Validator;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

/// A builder for creating forms with various fields.
pub struct FormBuilder {
    /// A map of field order to field name and field trait object.
    fields: BTreeMap<u32, (String, Box<dyn FieldTrait>)>,
    /// A counter to keep track of the order of fields.
    counter: u32,
}

impl FormBuilder {
    /// Creates a new `FormBuilder` instance.
    ///
    /// # Returns
    ///
    /// * A new `FormBuilder` instance.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
            counter: 0,
        }
    }

    /// Adds a field to the form.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `validator` - An optional `Validator` instance to validate the input.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the field value. It must implement the `FromStr`, `Debug`, `Clone`, and `Default` traits.
    ///
    /// # Returns
    ///
    /// * The `FormBuilder` instance with the added field.
    pub fn add_field<T>(mut self, name: &str, prompt: &str, validator: Option<Validator>) -> Self
    where
        T: 'static + FromStr + Debug + Clone + Default,
        T::Err: Debug,
    {
        self.fields.insert(
            self.counter,
            (
                name.to_string(),
                Box::new(Field::<T> {
                    prompt: prompt.to_string(),
                    validator,
                    value: None,
                }),
            ),
        );
        self.counter += 1;
        self
    }

    /// Adds a select field to the form.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `options` - A list of options available for selection.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the options. It must implement the `Clone`, `PartialEq`, `Debug`, and `FromStr` traits.
    ///
    /// # Returns
    ///
    /// * The `FormBuilder` instance with the added select field.
    pub fn add_select<T>(mut self, name: &str, prompt: &str, options: Vec<(T, &str)>) -> Self
    where
        T: 'static + Clone + PartialEq + Debug + FromStr,
        T::Err: Debug,
    {
        let options = options
            .into_iter()
            .map(|(v, s)| (v, s.to_string()))
            .collect();
        self.fields.insert(
            self.counter,
            (
                name.to_string(),
                Box::new(SelectField {
                    prompt: prompt.to_string(),
                    options,
                    value: None,
                }),
            ),
        );
        self.counter += 1;
        self
    }

    /// Adds a multiselect field to the form.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `options` - A list of options available for selection.
    /// * `limit` - An optional limit on the number of selections.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the options. It must implement the `Clone`, `PartialEq`, `Debug`, and `FromStr` traits.
    ///
    /// # Returns
    ///
    /// * The `FormBuilder` instance with the added multiselect field.
    pub fn add_multiselect<T>(
        mut self,
        name: &str,
        prompt: &str,
        options: Vec<(T, &str)>,
        limit: Option<usize>,
    ) -> Self
    where
        T: 'static + Clone + PartialEq + Debug + FromStr,
        T::Err: Debug,
    {
        let options = options
            .into_iter()
            .map(|(v, s)| (v, s.to_string()))
            .collect();
        self.fields.insert(
            self.counter,
            (
                name.to_string(),
                Box::new(MultiselectField {
                    prompt: prompt.to_string(),
                    options,
                    value: Vec::new(),
                    limit,
                }),
            ),
        );
        self.counter += 1;
        self
    }

    /// Builds the form and returns a `Form` instance.
    ///
    /// # Returns
    ///
    /// * A `Form` instance containing the added fields.
    pub fn build(self) -> Form {
        Form {
            fields: self.fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{ValidationMethods, Validator};
    use std::io::{BufRead, Cursor};

    /// Sets up a name validator for testing.
    ///
    /// # Returns
    ///
    /// * A `Validator` instance for validating names.
    fn setup_name_validator() -> Validator {
        Validator::new(vec![
            (ValidationMethods::validate_name, Some("Invalid name")),
            (ValidationMethods::not_empty, Some("Input cannot be empty")),
        ])
    }

    /// Sets up an email validator for testing.
    ///
    /// # Returns
    ///
    /// * A `Validator` instance for validating emails.
    fn setup_email_validator() -> Validator {
        Validator::new(vec![
            (ValidationMethods::validate_email, Some("Invalid email")),
            (ValidationMethods::not_empty, Some("Input cannot be empty")),
        ])
    }

    /// Reads input from a string for testing purposes.
    ///
    /// # Parameters
    ///
    /// * `_prompt` - The prompt message to be displayed to the user.
    /// * `validator` - An optional `Validator` instance to validate the input.
    /// * `input` - The input string to be read.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the input is successfully read and validated.
    /// * `Err(String)` if there is an error reading or validating the input.
    fn read_input(
        _prompt: &str,
        validator: Option<&Validator>,
        input: &str,
    ) -> Result<String, String> {
        let mut cursor = Cursor::new(input);
        let mut buffer = String::new();
        cursor.read_line(&mut buffer).unwrap();
        let trimmed_input = buffer.trim();
        if let Some(validator) = validator {
            validator.validate(trimmed_input)?;
        }
        Ok(trimmed_input.to_string())
    }

    #[test]
    fn test_read_input_valid_name() {
        let validator = setup_name_validator();
        let input = "John\n";
        let result = read_input("Enter name:", Some(&validator), input);
        assert_eq!(result, Ok("John".to_string()));
    }

    #[test]
    fn test_read_input_invalid_name() {
        let validator = setup_name_validator();
        let input = "John123\n";
        let result = read_input("Enter name:", Some(&validator), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_input_valid_email() {
        let validator = setup_email_validator();
        let input = "test@example.com\n";
        let result = read_input("Enter email:", Some(&validator), input);
        assert_eq!(result, Ok("test@example.com".to_string()));
    }

    #[test]
    fn test_read_input_invalid_email() {
        let validator = setup_email_validator();
        let input = "test@.com\n";
        let result = read_input("Enter email:", Some(&validator), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_input_not_empty() {
        let validator = setup_name_validator();
        let input = "non-empty\n";
        let result = read_input("Enter input:", Some(&validator), input);
        assert_eq!(result, Ok("non-empty".to_string()));
    }

    #[test]
    fn test_read_input_empty() {
        let validator = setup_name_validator();
        let input = "\n";
        let result = read_input("Enter input:", Some(&validator), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_field() {
        let form_builder = FormBuilder::new()
            .add_field::<String>("name", "Enter your name:", None)
            .add_field::<String>("email", "Enter your email:", None);
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 2);
    }

    #[test]
    fn test_add_select() {
        let form_builder = FormBuilder::new().add_select(
            "gender",
            "Select your gender:",
            vec![(1u32, "Male"), (2u32, "Female"), (3u32, "Other")],
        );
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_add_select_u32() {
        let form_builder = FormBuilder::new().add_select(
            "gender",
            "Select your gender:",
            vec![
                ("M".to_string(), "Male"),
                ("F".to_string(), "Female"),
                ("O".to_string(), "Other"),
            ],
        );
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_add_multiselect() {
        let form_builder = FormBuilder::new().add_multiselect(
            "hobbies",
            "Select your hobbies:",
            vec![
                ("reading".to_string(), "Reading"),
                ("sports".to_string(), "Sports"),
                ("music".to_string(), "Music"),
            ],
            Some(2),
        );
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_add_multiselect_u32() {
        let form_builder = FormBuilder::new().add_multiselect(
            "hobbies",
            "Select your hobbies:",
            vec![(1u32, "1"), (2u32, "2"), (3u32, "3")],
            Some(2),
        );
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 1);
    }
}
