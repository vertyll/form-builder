use crate::validation::Validator;
use std::str::FromStr;

pub fn read_input<T>(prompt: &str, validator: &Validator) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    loop {
        println!("{}", prompt);

        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let input = buffer.trim();

        if let Err(error_message) = validator.validate(input) {
            println!("{}", error_message);
            continue;
        }

        if let Ok(value) = input.parse::<T>() {
            return Ok(value);
        } else {
            println!("Failed to parse input. Please try again.");
            continue;
        }
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
