use crate::input::read_input;
use crate::validation::Validator;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

pub struct FormBuilder {
    fields: BTreeMap<String, Box<dyn FieldTrait>>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    pub fn add_field<T>(mut self, name: &str, prompt: &str, validator: Validator) -> Self
    where
        T: 'static + FromStr + Debug + Clone,
        T::Err: Debug,
    {
        self.fields.insert(
            name.to_string(),
            Box::new(Field::<T> {
                prompt: prompt.to_string(),
                validator,
                value: None,
            }),
        );
        self
    }

    pub fn build(self) -> Form {
        Form {
            fields: self.fields,
        }
    }
}

pub struct Form {
    fields: BTreeMap<String, Box<dyn FieldTrait>>,
}

impl Form {
    pub fn fill(&mut self) -> Result<(), String> {
        for (_name, field) in &mut self.fields {
            field.fill()?;
        }
        Ok(())
    }

    pub fn get_value<T>(&self, name: &str) -> Result<T, String>
    where
        T: 'static + FromStr + Debug + Clone,
        T::Err: Debug,
    {
        let field = self
            .fields
            .get(name)
            .ok_or_else(|| format!("Field '{}' not found", name))?;
        field
            .as_any()
            .downcast_ref::<Field<T>>()
            .ok_or_else(|| format!("Field '{}' has incorrect type", name))?
            .get_value()
    }
}

trait FieldTrait {
    fn fill(&mut self) -> Result<(), String>;
    fn as_any(&self) -> &dyn std::any::Any;
}

struct Field<T> {
    prompt: String,
    validator: Validator,
    value: Option<T>,
}

impl<T> FieldTrait for Field<T>
where
    T: 'static + FromStr + Debug + Clone,
    T::Err: Debug,
{
    fn fill(&mut self) -> Result<(), String> {
        loop {
            if let Ok(value) = read_input::<T>(&self.prompt, &self.validator) {
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
}

impl<T> Field<T>
where
    T: 'static + FromStr + Debug + Clone,
    T::Err: Debug,
{
    fn get_value(&self) -> Result<T, String> {
        self.value
            .as_ref()
            .ok_or_else(|| format!("Field has no value"))
            .map(|v| v.clone())
    }
}
