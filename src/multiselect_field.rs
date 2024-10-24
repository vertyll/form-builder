use crate::field::FieldTrait;
use crate::input::read_multiselect;
use std::fmt::Debug;
use std::str::FromStr;

/// A struct representing a multiselect field in a form.
#[derive(Debug)]
pub struct MultiselectField<T> {
    /// The prompt to display to the user.
    pub prompt: String,
    /// The options available for selection.
    pub options: Vec<(T, String)>,
    /// The selected values.
    pub value: Vec<T>,
    /// The optional limit on the number of selections.
    pub limit: Option<usize>,
}

impl<T> FieldTrait for MultiselectField<T>
where
    T: 'static + Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    /// Fills the multiselect field by prompting the user for input.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the field is successfully filled.
    /// * `Err(String)` if there is an error filling the field.
    fn fill(&mut self) -> Result<(), String> {
        // Używamy read_multiselect do odczytania wartości od użytkownika
        self.value = read_multiselect(&self.prompt, &self.options, self.limit)?;
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

    /// Gets the value of the multiselect field as a string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
    fn get_value(&self) -> Result<String, String> {
        // Return the value as a string
        Ok(format!("{:?}", self.value))
    }
}
