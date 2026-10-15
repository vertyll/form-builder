//! The form: built from fields by [`FormBuilder`], filled through a [`Console`], then read with [`Form::get`].

use std::any::{Any, type_name};
use std::fmt::{self, Display};
use std::str::FromStr;

use crate::Error;
use crate::console::{Console, Terminal};
use crate::selection::Mode;
use crate::validate::Validator;

type Value = Box<dyn Any>;
type Parse = Box<dyn Fn(&str) -> Result<Value, String>>;
type Pick = Box<dyn Fn(&[usize]) -> Value>;

/// What a field asks for, with what turns the answer into its value.
enum Kind {
    Text {
        validator: Option<Validator>,
        optional: bool,
        parse: Parse,
    },
    Choice {
        options: Vec<String>,
        mode: Mode,
        pick: Pick,
    },
}

struct Field {
    name: String,
    prompt: String,
    kind: Kind,
    value: Option<Value>,
}

impl Field {
    /// Whether the field needs an answer: every one but an optional text field.
    fn is_required(&self) -> bool {
        !matches!(self.kind, Kind::Text { optional: true, .. })
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("filled", &self.value.is_some())
            .finish_non_exhaustive()
    }
}

/// Collects the fields of a form, in the order they will be asked.
///
/// ```
/// use form_builder::FormBuilder;
/// use form_builder::validate::{self, Validator};
///
/// let form = FormBuilder::new()
///     .add_field::<String>("name", "Name:", Some(Validator::new().rule(validate::not_empty, "Required")))
///     .add_optional_field::<u32>("age", "Age (optional):", None)
///     .add_select("size", "Size:", [("S", "Small"), ("L", "Large")])
///     .build();
/// ```
///
/// The prompt of every required field is marked with `*`, before its closing colon: `Name:` is asked as `Name *:`.
/// [`required_marker`](Self::required_marker) changes the mark, and
/// [`no_required_marker`](Self::no_required_marker) leaves it out.
#[derive(Debug)]
#[must_use]
pub struct FormBuilder {
    fields: Vec<Field>,
    required_marker: Option<String>,
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            required_marker: Some("*".to_owned()),
        }
    }
}

impl FormBuilder {
    /// A builder with no fields, which marks required fields with `*`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a field read as text and parsed into a `T`. The trimmed input must pass `validator` first, and a value
    /// that does not parse is refused; either way the field asks again.
    pub fn add_field<T>(self, name: &str, prompt: &str, validator: Option<Validator>) -> Self
    where
        T: FromStr + 'static,
        T::Err: Display,
    {
        self.text::<T>(name, prompt, validator, false)
    }

    /// Adds a field like [`add_field`](Self::add_field) that may be left empty: its value is an `Option<T>`, `None`
    /// for an empty answer, which skips `validator`.
    pub fn add_optional_field<T>(self, name: &str, prompt: &str, validator: Option<Validator>) -> Self
    where
        T: FromStr + 'static,
        T::Err: Display,
    {
        self.text::<T>(name, prompt, validator, true)
    }

    /// Adds a field whose value is one of `options`, each a value and the label shown for it.
    pub fn add_select<T, L>(self, name: &str, prompt: &str, options: impl IntoIterator<Item = (T, L)>) -> Self
    where
        T: Clone + 'static,
        L: Into<String>,
    {
        let (values, labels) = split(options);
        let pick: Pick = Box::new(move |chosen| Box::new(values[chosen[0]].clone()));
        self.choice(name, prompt, labels, Mode::Single, pick)
    }

    /// Adds a field whose value is a `Vec<T>` of one or more of `options`, at most `limit` of them when given.
    pub fn add_multiselect<T, L>(
        self,
        name: &str,
        prompt: &str,
        options: impl IntoIterator<Item = (T, L)>,
        limit: Option<usize>,
    ) -> Self
    where
        T: Clone + 'static,
        L: Into<String>,
    {
        let (values, labels) = split(options);
        let pick: Pick =
            Box::new(move |chosen| Box::new(chosen.iter().map(|&index| values[index].clone()).collect::<Vec<T>>()));
        self.choice(name, prompt, labels, Mode::Multiple { limit }, pick)
    }

    /// Marks every required field with `marker` instead of `*`. A field is required unless it was added with
    /// [`add_optional_field`](Self::add_optional_field); a select or multiselect always needs an answer, so it is
    /// marked too.
    pub fn required_marker(mut self, marker: impl Into<String>) -> Self {
        self.required_marker = Some(marker.into());
        self
    }

    /// Leaves required fields unmarked, their prompts as given.
    pub fn no_required_marker(mut self) -> Self {
        self.required_marker = None;
        self
    }

    /// The form, ready to be filled.
    #[must_use]
    pub fn build(self) -> Form {
        Form {
            fields: self.fields,
            required_marker: self.required_marker,
        }
    }

    fn text<T>(mut self, name: &str, prompt: &str, validator: Option<Validator>, optional: bool) -> Self
    where
        T: FromStr + 'static,
        T::Err: Display,
    {
        let parse: Parse = if optional {
            Box::new(|input| match input {
                "" => Ok(Box::new(None::<T>)),
                _ => input
                    .parse::<T>()
                    .map(|value| Box::new(Some(value)) as Value)
                    .map_err(|e| e.to_string()),
            })
        } else {
            Box::new(|input| {
                input
                    .parse::<T>()
                    .map(|value| Box::new(value) as Value)
                    .map_err(|e| e.to_string())
            })
        };
        self.push(
            name,
            prompt,
            Kind::Text {
                validator,
                optional,
                parse,
            },
        );
        self
    }

    fn choice(mut self, name: &str, prompt: &str, options: Vec<String>, mode: Mode, pick: Pick) -> Self {
        self.push(name, prompt, Kind::Choice { options, mode, pick });
        self
    }

    fn push(&mut self, name: &str, prompt: &str, kind: Kind) {
        self.fields.push(Field {
            name: name.to_owned(),
            prompt: prompt.to_owned(),
            kind,
            value: None,
        });
    }
}

fn split<T, L: Into<String>>(options: impl IntoIterator<Item = (T, L)>) -> (Vec<T>, Vec<String>) {
    options.into_iter().map(|(value, label)| (value, label.into())).unzip()
}

/// A form of named fields, filled in order, whose values are then read by name and type.
#[derive(Debug)]
pub struct Form {
    fields: Vec<Field>,
    required_marker: Option<String>,
}

impl Form {
    /// Asks every field in the terminal, in order.
    ///
    /// # Errors
    ///
    /// Returns why a field could not be filled: the input ended, the user cancelled a choice, a choice has no
    /// options, or the terminal failed.
    pub fn fill(&mut self) -> Result<(), Error> {
        self.fill_with(&mut Terminal)
    }

    /// Asks every field through `console`, in order.
    ///
    /// # Errors
    ///
    /// As [`fill`](Self::fill).
    pub fn fill_with(&mut self, console: &mut impl Console) -> Result<(), Error> {
        for field in &mut self.fields {
            let prompt = match &self.required_marker {
                Some(marker) if field.is_required() => marked(&field.prompt, marker),
                _ => field.prompt.clone(),
            };
            field.value = Some(ask(field, &prompt, console)?);
        }
        Ok(())
    }

    /// The value of the field `name`: `T` for a field or a select, `Option<T>` for an optional field, `Vec<T>` for a
    /// multiselect.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when there is no such field, [`Error::NotFilled`] before it is filled, and
    /// [`Error::WrongType`] when it holds another type.
    pub fn get<T: Clone + 'static>(&self, name: &str) -> Result<T, Error> {
        let field = self
            .fields
            .iter()
            .find(|field| field.name == name)
            .ok_or_else(|| Error::NotFound(name.to_owned()))?;
        let value = field.value.as_ref().ok_or_else(|| Error::NotFilled(name.to_owned()))?;
        value.downcast_ref::<T>().cloned().ok_or_else(|| Error::WrongType {
            name: name.to_owned(),
            expected: type_name::<T>(),
        })
    }
}

/// `prompt` with `marker` before its closing colon, or after it when it has none.
fn marked(prompt: &str, marker: &str) -> String {
    let prompt = prompt.trim_end();
    match prompt.strip_suffix(':') {
        Some(stem) => format!("{stem} {marker}:"),
        None => format!("{prompt} {marker}"),
    }
}

fn ask(field: &Field, prompt: &str, console: &mut impl Console) -> Result<Value, Error> {
    match &field.kind {
        Kind::Text {
            validator,
            optional,
            parse,
        } => loop {
            let line = console
                .read_line(prompt)?
                .ok_or_else(|| Error::EndOfInput(field.name.clone()))?;
            let input = line.trim();
            let skip_checks = *optional && input.is_empty();
            if let Some(validator) = validator.as_ref().filter(|_| !skip_checks)
                && let Err(message) = validator.check(input)
            {
                console.report(message)?;
                continue;
            }
            match parse(input) {
                Ok(value) => return Ok(value),
                Err(reason) => console.report(&format!("Invalid input: {reason}"))?,
            }
        },
        Kind::Choice { options, mode, pick } => {
            if options.is_empty() {
                return Err(Error::NoOptions(field.name.clone()));
            }
            let chosen = console
                .choose(prompt, options, *mode)?
                .ok_or_else(|| Error::Cancelled(field.name.clone()))?;
            Ok(pick(&chosen))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::marked;

    #[test]
    fn the_marker_goes_before_the_closing_colon() {
        assert_eq!(marked("Name:", "*"), "Name *:");
        assert_eq!(marked("Name: ", "*"), "Name *:");
        assert_eq!(marked("Your name", "*"), "Your name *");
        assert_eq!(marked("Email:", "(required)"), "Email (required):");
    }
}
