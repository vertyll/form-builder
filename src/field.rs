use crate::input::read_input;
use crate::validation::Validator;
use std::fmt::Debug;
use std::str::FromStr;

/// A trait for form fields.
pub trait FieldTrait {
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

    /// Gets the value of the field as a string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
    fn get_value(&self) -> Result<String, String>;
}

/// A struct representing a form field.
#[derive(Debug)]
pub struct Field<T> {
    /// The prompt to display to the user.
    pub prompt: String,
    /// An optional validator for the field.
    pub validator: Option<Validator>,
    /// The value of the field.
    pub value: Option<T>,
}

impl<T> FieldTrait for Field<T>
where
    T: 'static + FromStr + Debug + Clone + Default,
    T::Err: Debug,
{
    /// Fills the field by prompting the user for input.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the field is successfully filled.
    /// * `Err(String)` if there is an error filling the field.
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

    /// Returns a reference to the field as a `dyn Any`.
    ///
    /// # Returns
    ///
    /// * A reference to the field as a `dyn Any`.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Gets the value of the field as a string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
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
    pub fn get_value(&self) -> Result<T, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| v.clone())
    }
}
