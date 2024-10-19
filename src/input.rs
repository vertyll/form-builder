use crate::validation::Validator;
use std::fmt::Debug;
use std::str::FromStr;

/// Reads input from the user and validates it using the provided validator.
///
/// # Arguments
///
/// * `prompt` - The prompt message to be displayed to the user.
/// * `validator` - An optional `Validator` instance to validate the input.
///
/// # Type Parameters
///
/// * `T` - The type of the input value. It must implement the `FromStr` and `Debug` traits.
///
/// # Returns
///
/// * `Ok(T)` if the input is successfully read and validated.
/// * `Err(String)` if there is an error reading or validating the input.
pub fn read_input<T>(prompt: &str, validator: Option<&Validator>) -> Result<T, String>
where
    T: FromStr,
    T::Err: Debug,
{
    use std::io::{self, Write};

    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Some(validator) = validator {
            if let Err(err) = validator.validate(input) {
                println!("{}", err);
                continue;
            }
        }

        return input.parse::<T>().map_err(|err| format!("{:?}", err));
    }
}
