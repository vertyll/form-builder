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

    /// Validates that the value has a minimum length.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    /// * `min_length` - The minimum length of the value.
    ///
    /// # Returns
    ///
    /// * `true` if the value has at least `min_length` characters, `false` otherwise.
    pub fn min_length(value: &str, min_length: usize) -> bool {
        value.len() >= min_length
    }

    /// Validates that the value has a maximum length.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    /// * `max_length` - The maximum length of the value.
    ///
    /// # Returns
    ///
    /// * `true` if the value has at most `max_length` characters, `false` otherwise.
    pub fn max_length(value: &str, max_length: usize) -> bool {
        value.len() <= max_length
    }

    /// Validates that the value contains only alphabetic characters.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value contains only alphabetic characters, `false` otherwise.
    pub fn is_alpha(value: &str) -> bool {
        value.chars().all(|c| c.is_alphabetic())
    }

    /// Validates that the value is an integer.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is an integer, `false` otherwise.
    pub fn is_integer(value: &str) -> bool {
        value.parse::<i32>().is_ok()
    }

    /// Validates that the value is a floating-point number.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is a floating-point number, `false` otherwise.
    pub fn is_float(value: &str) -> bool {
        value.parse::<f64>().is_ok()
    }

    /// Validates that the value is in a date format (YYYY-MM-DD).
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a date format, `false` otherwise.
    pub fn is_date(value: &str) -> bool {
        let date_regex = Regex::new(r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$").unwrap();
        date_regex.is_match(value)
    }

    /// Validates that the value is in a time format (HH:MM:SS).
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a time format, `false` otherwise.
    pub fn is_time(value: &str) -> bool {
        let time_regex = Regex::new(r"^(0[0-9]|1[0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]$").unwrap();
        time_regex.is_match(value)
    }

    /// Validates that the value is in a URL format.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a URL format, `false` otherwise.
    pub fn is_url(value: &str) -> bool {
        let url_regex = Regex::new(r"^(http|https)://[^\s/$.?#].[^\s]*$").unwrap();
        url_regex.is_match(value)
    }

    /// Validates that the value is in a phone number format.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a phone number format, `false` otherwise.
    pub fn is_phone_number(value: &str) -> bool {
        let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
        phone_regex.is_match(value)
    }

    /// Validates that the value is in a postal code format (e.g., 12345 or 12345-6789).
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a postal code format, `false` otherwise.
    pub fn is_postal_code(value: &str) -> bool {
        let postal_code_regex = Regex::new(r"^\d{5}(-\d{4})?$").unwrap();
        postal_code_regex.is_match(value)
    }

    /// Validates that the value is in a credit card number format.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a credit card number format, `false` otherwise.
    pub fn is_credit_card(value: &str) -> bool {
        let credit_card_regex = Regex::new(r"^\d{4}-?\d{4}-?\d{4}-?\d{4}$").unwrap();
        credit_card_regex.is_match(value)
    }

    /// Validates that the value is in a UUID format.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the value to be validated.
    ///
    /// # Returns
    ///
    /// * `true` if the value is in a UUID format, `false` otherwise.
    pub fn is_uuid(value: &str) -> bool {
        let uuid_regex = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .unwrap();
        uuid_regex.is_match(value)
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

    #[test]
    fn test_min_length() {
        assert!(ValidationMethods::min_length("hello", 3));
        assert!(!ValidationMethods::min_length("hi", 3));
    }

    #[test]
    fn test_max_length() {
        assert!(ValidationMethods::max_length("hello", 10));
        assert!(!ValidationMethods::max_length("hello world", 10));
    }

    #[test]
    fn test_is_alpha() {
        assert!(ValidationMethods::is_alpha("hello"));
        assert!(!ValidationMethods::is_alpha("hello123"));
    }

    #[test]
    fn test_is_integer() {
        assert!(ValidationMethods::is_integer("123"));
        assert!(!ValidationMethods::is_integer("123abc"));
    }

    #[test]
    fn test_is_float() {
        assert!(ValidationMethods::is_float("123.45"));
        assert!(!ValidationMethods::is_float("123.45abc"));
    }

    #[test]
    fn test_is_date() {
        assert!(ValidationMethods::is_date("2023-10-01"));
        assert!(!ValidationMethods::is_date("2023-10-32"));
    }

    #[test]
    fn test_is_time() {
        assert!(ValidationMethods::is_time("12:34:56"));
        assert!(!ValidationMethods::is_time("25:34:56"));
    }

    #[test]
    fn test_is_url() {
        assert!(ValidationMethods::is_url("https://example.com"));
        assert!(!ValidationMethods::is_url("htp://example.com"));
    }

    #[test]
    fn test_is_phone_number() {
        assert!(ValidationMethods::is_phone_number("+1234567890"));
        assert!(!ValidationMethods::is_phone_number("123-456-7890"));
    }

    #[test]
    fn test_is_postal_code() {
        assert!(ValidationMethods::is_postal_code("12345"));
        assert!(ValidationMethods::is_postal_code("12345-6789"));
        assert!(!ValidationMethods::is_postal_code("1234"));
    }

    #[test]
    fn test_is_credit_card() {
        assert!(ValidationMethods::is_credit_card("1234-5678-1234-5678"));
        assert!(ValidationMethods::is_credit_card("1234567812345678"));
        assert!(!ValidationMethods::is_credit_card("1234-5678-1234-567"));
    }

    #[test]
    fn test_is_uuid() {
        assert!(ValidationMethods::is_uuid(
            "123e4567-e89b-12d3-a456-426614174000"
        ));
        assert!(!ValidationMethods::is_uuid(
            "123e4567-e89b-12d3-a456-42661417400"
        ));
    }
}
