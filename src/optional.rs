use std::fmt::{self, Debug, Display};
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

/// An optional value that can be `Some` or `None`.
#[derive(Debug, Clone)]
pub enum Optional<T> {
    Some(T),
    None,
}

impl<T> FromStr for Optional<T>
where
    T: FromStr,
    T::Err: Debug,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Optional::None)
        } else {
            s.parse::<T>().map(Optional::Some)
        }
    }
}

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Optional::None
    }
}

impl<T> Display for Optional<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Optional::Some(value) => write!(f, "{}", value),
            Optional::None => write!(f, "None"),
        }
    }
}

impl<T> Add for Optional<T>
where
    T: Add<Output = T> + Default,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some(a + b),
            (Optional::Some(a), Optional::None) => Optional::Some(a),
            (Optional::None, Optional::Some(b)) => Optional::Some(b),
            (Optional::None, Optional::None) => Optional::None,
        }
    }
}

impl<T> Sub for Optional<T>
where
    T: Sub<Output = T> + Default,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some(a - b),
            (Optional::Some(a), Optional::None) => Optional::Some(a),
            (Optional::None, Optional::Some(b)) => Optional::Some(b),
            (Optional::None, Optional::None) => Optional::None,
        }
    }
}

impl<T> Mul for Optional<T>
where
    T: Mul<Output = T> + Default,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some(a * b),
            (Optional::Some(a), Optional::None) => Optional::Some(a),
            (Optional::None, Optional::Some(b)) => Optional::Some(b),
            (Optional::None, Optional::None) => Optional::None,
        }
    }
}

impl<T> Div for Optional<T>
where
    T: Div<Output = T> + Default,
{
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some(a / b),
            (Optional::Some(a), Optional::None) => Optional::Some(a),
            (Optional::None, Optional::Some(b)) => Optional::Some(b),
            (Optional::None, Optional::None) => Optional::None,
        }
    }
}
