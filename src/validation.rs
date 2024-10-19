use regex::Regex;

/// A struct containing various validation methods.
pub struct ValidationMethods;

impl ValidationMethods {
    /// Validates that the name does not contain any numeric characters.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the name does not contain numeric characters, `false` otherwise.
    pub fn validate_name(name: &str) -> bool {
        !name.chars().any(|c| c.is_numeric())
    }

    /// Validates that the email is in a proper format.
    ///
    /// # Arguments
    ///
    /// * `email` - A string slice that holds the email to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the email is in a valid format, `false` otherwise.
    pub fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        email_regex.is_match(email)
    }

    /// Validates that the value is not empty.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is not empty, `false` otherwise.
    pub fn not_empty(value: &str) -> bool {
        !value.is_empty()
    }
}

/// A struct that holds a list of validation functions and their corresponding error messages.
#[derive(Debug)]
pub struct Validator {
    pub validations: Vec<(fn(&str) -> bool, Option<&'static str>)>,
}

impl Validator {
    /// Creates a new `Validator` with the given list of validation functions and error messages.
    ///
    /// # Arguments
    ///
    /// * `validations` - A vector of tuples where each tuple contains a validation function and an optional error message.
    ///
    /// # Returns
    ///
    /// * A new instance of `Validator`.
    pub fn new(validations: Vec<(fn(&str) -> bool, Option<&'static str>)>) -> Self {
        Self { validations }
    }

    /// Validates the input string using the list of validation functions.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice that holds the input to be validated.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the input passes all validations, `Err(String)` with an error message otherwise.
    pub fn validate(&self, input: &str) -> Result<(), String> {
        for (validation, error_message) in &self.validations {
            if !validation(input) {
                return Err(error_message
                    .unwrap_or("Invalid input, please try again.")
                    .to_string());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ValidationMethods;

    #[test]
    fn test_validate_name() {
        assert!(ValidationMethods::validate_name("John"));
        assert!(!ValidationMethods::validate_name("John123"));
    }

    #[test]
    fn test_validate_email() {
        assert!(ValidationMethods::validate_email("test@example.com"));
        assert!(!ValidationMethods::validate_email("test@.com"));
        assert!(!ValidationMethods::validate_email("test@com"));
    }

    #[test]
    fn test_not_empty() {
        assert!(ValidationMethods::not_empty("non-empty"));
        assert!(!ValidationMethods::not_empty(""));
    }
}