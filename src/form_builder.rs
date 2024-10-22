use crate::input::read_input;
use crate::optional::Optional;
use crate::validation::Validator;
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt::Debug;
use std::str::FromStr;

/// A builder for creating forms with various fields.
pub struct FormBuilder {
    fields: BTreeMap<u32, (String, Box<dyn FieldTrait>)>,
    counter: u32,
}

impl FormBuilder {
    /// Creates a new `FormBuilder` instance.
    ///
    /// # Returns
    ///
    /// * A new instance of `FormBuilder`.
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
            counter: 0,
        }
    }

    /// Adds a field to the form.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `validator` - An optional `Validator` instance to validate the field input.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the field value. It must implement the `FromStr`, `Debug`, `Clone`, and `Default` traits.
    ///
    /// # Returns
    ///
    /// * The updated `FormBuilder` instance.
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

    /// Builds the form.
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

/// A struct representing a form with various fields.
pub struct Form {
    fields: BTreeMap<u32, (String, Box<dyn FieldTrait>)>,
}

impl Form {
    /// Fills the form by prompting the user for input for each field.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all fields are successfully filled.
    /// * `Err(String)` if there is an error filling any field.
    pub fn fill(&mut self) -> Result<(), String> {
        for (_order, (_name, field)) in &mut self.fields {
            field.fill()?;
        }
        Ok(())
    }

    /// Gets the value of a field.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the field.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the field value. It must implement the `FromStr`, `Debug`, `Clone`, and `Default` traits.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field is not found or if the field type is incorrect.
    pub fn get_value<T>(&self, name: &str) -> Result<T, String>
    where
        T: 'static + FromStr + Debug + Clone + Default,
        T::Err: Debug,
    {
        let field = self
            .fields
            .values()
            .find(|(field_name, _)| field_name == name)
            .ok_or_else(|| format!("Field '{}' not found", name))?
            .1
            .as_ref();

        if let Some(field) = field.as_any().downcast_ref::<Field<T>>() {
            field.get_value()
        } else if let Some(field) = field.as_any().downcast_ref::<Field<Optional<T>>>() {
            field.get_value().map(|opt| match opt {
                Optional::Some(value) => value,
                Optional::None => T::default(),
            })
        } else {
            Err(format!("Field '{}' has incorrect type", name))
        }
    }
}

/// A trait for form fields.
trait FieldTrait {
    /// Fills the field by prompting the user for input.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the field is successfully filled.
    /// * `Err(String)` if there is an error filling the field.
    fn fill(&mut self) -> Result<(), String>;

    /// Returns a reference to the field as a `dyn Any`.
    ///
    /// # Returns
    ///
    /// * A reference to the field as a `dyn Any`.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// A struct representing a form field.
#[derive(Debug)]
struct Field<T> {
    prompt: String,
    validator: Option<Validator>,
    value: Option<T>,
}

impl<T> FieldTrait for Field<T>
where
    T: 'static + FromStr + Debug + Clone + Default,
    T::Err: Debug,
{
    fn fill(&mut self) -> Result<(), String> {
        loop {
            if let Ok(value) = read_input::<T>(&self.prompt, self.validator.as_ref()) {
                self.value = Some(value);
                break;
            } else {
                println!("Invalid input. Please try again.");
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T> Field<T>
where
    T: 'static + FromStr + Debug + Clone,
    T::Err: Debug,
{
    /// Gets the value of the field.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
    fn get_value(&self) -> Result<T, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| v.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{ValidationMethods, Validator};
    use std::io::{BufRead, Cursor};

    fn setup_name_validator() -> Validator {
        Validator::new(vec![
            (ValidationMethods::validate_name, Some("Invalid name")),
            (ValidationMethods::not_empty, Some("Input cannot be empty")),
        ])
    }

    fn setup_email_validator() -> Validator {
        Validator::new(vec![
            (ValidationMethods::validate_email, Some("Invalid email")),
            (ValidationMethods::not_empty, Some("Input cannot be empty")),
        ])
    }

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
}
