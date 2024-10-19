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
