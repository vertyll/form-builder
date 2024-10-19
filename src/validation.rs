use regex::Regex;

pub struct ValidationMethods;

impl ValidationMethods {
    pub fn validate_name(name: &str) -> bool {
        !name.chars().any(|c| c.is_numeric())
    }

    pub fn validate_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        email_regex.is_match(email)
    }

    pub fn not_empty(value: &str) -> bool {
        !value.is_empty()
    }
}

pub struct Validator {
    pub validations: Vec<(fn(&str) -> bool, Option<&'static str>)>,
}

impl Validator {
    pub fn new(validations: Vec<(fn(&str) -> bool, Option<&'static str>)>) -> Self {
        Self { validations }
    }

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