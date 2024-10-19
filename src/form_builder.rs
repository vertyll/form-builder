use crate::input::read_input;
use crate::validation::Validator;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

pub struct FormBuilder {
    fields: BTreeMap<String, Box<dyn FieldTrait>>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    pub fn add_field<T>(mut self, name: &str, prompt: &str, validator: Validator) -> Self
    where
        T: 'static + FromStr + Debug + Clone,
        T::Err: Debug,
    {
        self.fields.insert(
            name.to_string(),
            Box::new(Field::<T> {
                prompt: prompt.to_string(),
                validator,
                value: None,
            }),
        );
        self
    }

    pub fn build(self) -> Form {
        Form {
            fields: self.fields,
        }
    }
}

pub struct Form {
    fields: BTreeMap<String, Box<dyn FieldTrait>>,
}

impl Form {
    pub fn fill(&mut self) -> Result<(), String> {
        for (_name, field) in &mut self.fields {
            field.fill()?;
        }
        Ok(())
    }

    pub fn get_value<T>(&self, name: &str) -> Result<T, String>
    where
        T: 'static + FromStr + Debug + Clone,
        T::Err: Debug,
    {
        let field = self
            .fields
            .get(name)
            .ok_or_else(|| format!("Field '{}' not found", name))?;
        field
            .as_any()
            .downcast_ref::<Field<T>>()
            .ok_or_else(|| format!("Field '{}' has incorrect type", name))?
            .get_value()
    }
}

trait FieldTrait {
    fn fill(&mut self) -> Result<(), String>;
    fn as_any(&self) -> &dyn std::any::Any;
}

struct Field<T> {
    prompt: String,
    validator: Validator,
    value: Option<T>,
}

impl<T> FieldTrait for Field<T>
where
    T: 'static + FromStr + Debug + Clone,
    T::Err: Debug,
{
    fn fill(&mut self) -> Result<(), String> {
        loop {
            if let Ok(value) = read_input::<T>(&self.prompt, &self.validator) {
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
    fn get_value(&self) -> Result<T, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| v.clone())
    }
}

#[cfg(test)]
mod tests {
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

    fn read_input(_prompt: &str, validator: &Validator, input: &str) -> Result<String, String> {
        let mut cursor = Cursor::new(input);
        let mut buffer = String::new();
        cursor.read_line(&mut buffer).unwrap();
        let trimmed_input = buffer.trim();
        validator.validate(trimmed_input)?;
        Ok(trimmed_input.to_string())
    }

    #[test]
    fn test_read_input_valid_name() {
        let validator = setup_name_validator();
        let input = "John\n";
        let result = read_input("Enter name:", &validator, input);
        assert_eq!(result, Ok("John".to_string()));
    }

    #[test]
    fn test_read_input_invalid_name() {
        let validator = setup_name_validator();
        let input = "John123\n";
        let result = read_input("Enter name:", &validator, input);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_input_valid_email() {
        let validator = setup_email_validator();
        let input = "test@example.com\n";
        let result = read_input("Enter email:", &validator, input);
        assert_eq!(result, Ok("test@example.com".to_string()));
    }

    #[test]
    fn test_read_input_invalid_email() {
        let validator = setup_email_validator();
        let input = "test@.com\n";
        let result = read_input("Enter email:", &validator, input);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_input_not_empty() {
        let validator = setup_name_validator();
        let input = "non-empty\n";
        let result = read_input("Enter input:", &validator, input);
        assert_eq!(result, Ok("non-empty".to_string()));
    }

    #[test]
    fn test_read_input_empty() {
        let validator = setup_name_validator();
        let input = "\n";
        let result = read_input("Enter input:", &validator, input);
        assert!(result.is_err());
    }
}
