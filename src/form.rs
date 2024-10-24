use crate::field::{Field, FieldTrait};
use crate::multiselect_field::MultiselectField;
use crate::optional::Optional;
use crate::select_field::SelectField;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

/// A struct representing a form with multiple fields.
pub struct Form {
    /// A map of field order to field name and field trait object.
    pub fields: BTreeMap<u32, (String, Box<dyn FieldTrait>)>,
}

impl Form {
    /// Fills all fields in the form by prompting the user for input.
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

    /// Gets the value of a field by its name.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the field.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field is not found or has an incorrect type.
    pub fn get_value<T>(&self, name: &str) -> Result<T, String>
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

    /// Gets the value of a field as a vector by its name.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the field.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<T>)` if the field value is successfully retrieved.
    /// * `Err(String)` if the field is not found or has an incorrect type.
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
            Ok(vec![field.get_value()?])
        } else if let Some(field) = field.as_any().downcast_ref::<Field<Optional<T>>>() {
            let value = field.get_value()?;
            match value {
                Optional::Some(v) => Ok(vec![v]),
                Optional::None => Ok(vec![T::default()]),
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
