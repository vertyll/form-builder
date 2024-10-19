use std::fmt::{self, Debug, Display};
use std::str::FromStr;

/// A wrapper type for `Option<T>` that implements `FromStr`.
#[derive(Debug, Clone)]
pub struct Optional<T>(pub Option<T>);

impl<T> FromStr for Optional<T>
where
    T: FromStr,
    T::Err: Debug,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Optional(None))
        } else {
            s.parse::<T>().map(|value| Optional(Some(value)))
        }
    }
}

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Optional(None)
    }
}

impl<T> Display for Optional<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, "None"),
        }
    }
}
