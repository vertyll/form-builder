use crate::input::read_input;
use crate::optional::Optional;
use crate::validation::Validator;
use libc::{tcgetattr, tcsetattr, ECHO, ICANON, TCSANOW};
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt::Debug;
use std::io::stdin;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
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

    /// Adds a select field to the form.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `options` - A vector of options for the select field.
    ///
    /// # Returns
    ///
    /// * The updated `FormBuilder` instance.
    pub fn add_select<T>(mut self, name: &str, prompt: &str, options: Vec<(T, String)>) -> Self
    where
        T: 'static + Clone + PartialEq + Debug + FromStr,
        T::Err: Debug,
    {
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
    /// # Arguments
    ///
    /// * `name` - The name of the field.
    /// * `prompt` - The prompt message to be displayed to the user.
    /// * `options` - A vector of options for the multiselect field.
    /// * `limit` - An optional limit for the number of options that can be selected.
    ///
    /// # Returns
    ///
    /// * The updated `FormBuilder` instance.
    pub fn add_multiselect<T>(
        mut self,
        name: &str,
        prompt: &str,
        options: Vec<(T, String)>,
        limit: Option<usize>,
    ) -> Self
    where
        T: 'static + Clone + PartialEq + Debug + FromStr,
        T::Err: Debug,
    {
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
        T: 'static + FromStr + Debug + Clone + Default + PartialEq, // Add PartialEq here
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
        } else if let Some(field) = field.as_any().downcast_ref::<SelectField<T>>() {
            field
                .get_value()
                .and_then(|v| v.parse::<T>().map_err(|e| format!("{:?}", e)))
        } else if let Some(field) = field.as_any().downcast_ref::<MultiselectField<T>>() {
            field
                .get_value()
                .and_then(|v| v.parse::<T>().map_err(|e| format!("{:?}", e)))
        } else {
            Err(format!("Field '{}' has incorrect type", name))
        }
    }

    pub fn get_value_vec<T>(&self, name: &str) -> Result<Vec<T>, String>
    where
        T: 'static + FromStr + Debug + Clone + Default + PartialEq,
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
            Ok(vec![field.get_value()?]) // Wrap single value in a Vec
        } else if let Some(field) = field.as_any().downcast_ref::<Field<Optional<T>>>() {
            let value = field.get_value()?;
            match value {
                Optional::Some(v) => Ok(vec![v]),
                Optional::None => Ok(vec![T::default()]), // Or return an empty Vec based on your needs
            }
        } else if let Some(field) = field.as_any().downcast_ref::<SelectField<T>>() {
            let value = field
                .get_value()
                .and_then(|v| v.parse::<T>().map_err(|e| format!("{:?}", e)))?;
            Ok(vec![value])
        } else if let Some(field) = field.as_any().downcast_ref::<MultiselectField<T>>() {
            let value = field.get_value()?;
            let value = value.trim_matches(|c| c == '[' || c == ']').to_string();
            let values: Result<Vec<T>, _> = value
                .split(',')
                .map(|s| s.trim_matches(|c| c == '"' || c == ' ').parse::<T>())
                .collect();
            values.map_err(|e| format!("{:?}", e))
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

    /// Gets the value of the field.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
    fn get_value(&self) -> Result<String, String>;
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

    fn get_value(&self) -> Result<String, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| format!("{:?}", v))
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

/// A struct representing a select field.
#[derive(Debug)]
struct SelectField<T> {
    prompt: String,
    options: Vec<(T, String)>, // Dowolna wartość T + opis w String
    value: Option<T>,
}

impl<T> FieldTrait for SelectField<T>
where
    T: 'static + Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    fn fill(&mut self) -> Result<(), String> {
        self.value = Some(read_select::<T>(&self.prompt, &self.options)?);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_value(&self) -> Result<String, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| format!("{:?}", v))
    }
}

/// A struct representing a multiselect field.
#[derive(Debug)]
struct MultiselectField<T> {
    prompt: String,
    options: Vec<(T, String)>,
    value: Vec<T>,
    limit: Option<usize>,
}

impl<T> FieldTrait for MultiselectField<T>
where
    T: 'static + Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    fn fill(&mut self) -> Result<(), String> {
        self.value = read_multiselect(&self.prompt, &self.options, self.limit)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_value(&self) -> Result<String, String> {
        Ok(format!("{:?}", self.value))
    }
}

enum Key {
    Up,
    Down,
    Enter,
    Space,
    Other,
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    if io::stdout().flush().is_err() {
        eprintln!("Failed to flush stdout");
    }
}

fn read_key_raw() -> Result<Key, String> {
    let stdin_fd = stdin().as_raw_fd();
    let mut termios = unsafe { std::mem::zeroed() };
    if unsafe { tcgetattr(stdin_fd, &mut termios) } < 0 {
        return Err("Failed to get terminal attributes".to_string());
    }

    termios.c_lflag &= !(ICANON | ECHO);
    if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios) } < 0 {
        return Err("Failed to set terminal attributes".to_string());
    }

    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer).unwrap();

    termios.c_lflag |= ICANON | ECHO;
    if unsafe { tcsetattr(stdin_fd, TCSANOW, &termios) } < 0 {
        return Err("Failed to set terminal attributes".to_string());
    }

    match buffer[0] {
        65 => Ok(Key::Up),
        66 => Ok(Key::Down),
        10 => Ok(Key::Enter),
        32 => Ok(Key::Space),
        _ => Ok(Key::Other),
    }
}

fn read_select<T>(prompt: &str, options: &[(T, String)]) -> Result<T, String>
where
    T: Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    let mut selected = 0;

    loop {
        clear_screen();
        println!("{}:", prompt);

        for (i, (_, value)) in options.iter().enumerate() {
            if i == selected {
                println!("> {}", value);
            } else {
                println!("  {}", value);
            }
        }
        if io::stdout().flush().is_err() {
            eprintln!("Failed to flush stdout");
        }

        match read_key_raw()? {
            Key::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Key::Down => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            Key::Enter => {
                clear_screen();
                return Ok(options[selected].0.clone());
            }
            _ => {}
        }
    }
}

fn read_multiselect<T>(
    prompt: &str,
    options: &[(T, String)],
    limit: Option<usize>,
) -> Result<Vec<T>, String>
where
    T: Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    let mut selected = 0;
    let mut selected_options = vec![false; options.len()];

    loop {
        clear_screen();
        println!("{}:", prompt);
        println!("Use Space to select/deselect, Enter to confirm");

        for (i, (_, value)) in options.iter().enumerate() {
            let marker = if selected_options[i] { "*" } else { " " };
            if i == selected {
                println!("> [{}] {}", marker, value);
            } else {
                println!("  [{}] {}", marker, value);
            }
        }
        if io::stdout().flush().is_err() {
            eprintln!("Failed to flush stdout");
        }

        match read_key_raw()? {
            Key::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Key::Down => {
                if selected < options.len() - 1 {
                    selected += 1;
                }
            }
            Key::Space => {
                if selected_options[selected] {
                    selected_options[selected] = false;
                } else if limit.is_none()
                    || selected_options.iter().filter(|&&x| x).count() < limit.unwrap()
                {
                    selected_options[selected] = true;
                }
            }
            Key::Enter => {
                let selected_keys: Vec<T> = options
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| selected_options[*i])
                    .map(|(_, (key, _))| key.clone())
                    .collect();

                if !selected_keys.is_empty() {
                    clear_screen();
                    return Ok(selected_keys);
                }
            }
            _ => {}
        }
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

    #[test]
    fn test_add_select() {
        let form_builder = FormBuilder::new().add_select(
            "gender",
            "Select your gender:",
            vec![
                ("M".to_string(), "Male".to_string()),
                ("F".to_string(), "Female".to_string()),
                ("O".to_string(), "Other".to_string()),
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
                ("reading".to_string(), "Reading".to_string()),
                ("sports".to_string(), "Sports".to_string()),
                ("music".to_string(), "Music".to_string()),
            ],
            Some(2),
        );
        let form = form_builder.build();
        assert_eq!(form.fields.len(), 1);
    }
}
