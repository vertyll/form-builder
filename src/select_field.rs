use crate::field::FieldTrait;
use crate::input::read_select;
use std::fmt::Debug;
use std::str::FromStr;

/// A struct representing a select field in a form.
#[derive(Debug)]
pub struct SelectField<T> {
    /// The prompt to display to the user.
    pub prompt: String,
    /// The options available for selection.
    pub options: Vec<(T, String)>,
    /// The selected value.
    pub value: Option<T>,
}

impl<T> FieldTrait for SelectField<T>
where
    T: 'static + Clone + PartialEq + Debug + FromStr,
    T::Err: Debug,
{
    /// Fills the select field by prompting the user for input.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the field is successfully filled.
    /// * `Err(String)` if there is an error filling the field.
    fn fill(&mut self) -> Result<(), String> {
        // Use the read_select function to prompt the user for input
        self.value = Some(read_select::<T>(&self.prompt, &self.options)?);
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

    /// Gets the value of the select field as a string.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field has no value.
    fn get_value(&self) -> Result<String, String> {
        // Return the value of the field as a string
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| format!("{:?}", v))
    }
}
