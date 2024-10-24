use std::fmt::{self, Debug, Display};
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

/// An optional value that can be `Some` or `None`.
#[derive(Debug, Clone)]
pub enum Optional<T> {
    /// Contains a value of type `T`.
    Some(T),
    /// Represents the absence of a value.
    None,
}

impl<T> FromStr for Optional<T>
where
    T: FromStr,
    T::Err: Debug,
{
    type Err = T::Err;

    /// Parses a string `s` to produce an `Optional<T>`.
    ///
    /// # Returns
    ///
    /// * `Ok(Optional::None)` if the string is empty.
    /// * `Ok(Optional::Some(T))` if the string can be parsed to `T`.
    /// * `Err(T::Err)` if the string cannot be parsed to `T`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Optional::None)
        } else {
            s.parse::<T>().map(Optional::Some)
        }
    }
}

impl<T> Default for Optional<T> {
    /// Returns the default value for `Optional<T>`, which is `Optional::None`.
    fn default() -> Self {
        Optional::None
    }
}

impl<T: PartialEq> PartialEq for Optional<T> {
    /// Checks if two `Optional<T>` values are equal.
    ///
    /// # Returns
    ///
    /// * `true` if both are `Optional::None` or both are `Optional::Some` with equal values.
    /// * `false` otherwise.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Optional::None, Optional::None) => true,
            (Optional::Some(a), Optional::Some(b)) => a == b,
            _ => false,
        }
    }
}

impl<T> Display for Optional<T>
where
    T: Display,
{
    /// Formats the value using the given formatter.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the value is successfully formatted.
    /// * `Err(fmt::Error)` if there is an error formatting the value.
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

    /// Adds two `Optional<T>` values.
    ///
    /// # Returns
    ///
    /// * `Optional::Some(a + b)` if both are `Optional::Some`.
    /// * `Optional::Some(a)` if one is `Optional::None`.
    /// * `Optional::None` if both are `Optional::None`.
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

    /// Subtracts two `Optional<T>` values.
    ///
    /// # Returns
    ///
    /// * `Optional::Some(a - b)` if both are `Optional::Some`.
    /// * `Optional::Some(a)` if one is `Optional::None`.
    /// * `Optional::None` if both are `Optional::None`.
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

    /// Multiplies two `Optional<T>` values.
    ///
    /// # Returns
    ///
    /// * `Optional::Some(a * b)` if both are `Optional::Some`.
    /// * `Optional::Some(a)` if one is `Optional::None`.
    /// * `Optional::None` if both are `Optional::None`.
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

    /// Divides two `Optional<T>` values.
    ///
    /// # Returns
    ///
    /// * `Optional::Some(a / b)` if both are `Optional::Some`.
    /// * `Optional::Some(a)` if one is `Optional::None`.
    /// * `Optional::None` if both are `Optional::None`.
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Optional::Some(a), Optional::Some(b)) => Optional::Some(a / b),
            (Optional::Some(a), Optional::None) => Optional::Some(a),
            (Optional::None, Optional::Some(b)) => Optional::Some(b),
            (Optional::None, Optional::None) => Optional::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str() {
        assert_eq!(Optional::<i32>::from_str("").unwrap(), Optional::None);
        assert_eq!(Optional::<i32>::from_str("42").unwrap(), Optional::Some(42));
    }

    #[test]
    fn test_default() {
        let default_value: Optional<i32> = Default::default();
        assert_eq!(default_value, Optional::None);
    }

    #[test]
    fn test_display() {
        let some_value = Optional::Some(42);
        let none_value: Optional<i32> = Optional::None;
        assert_eq!(format!("{}", some_value), "42");
        assert_eq!(format!("{}", none_value), "None");
    }

    #[test]
    fn test_add() {
        let a = Optional::Some(2);
        let b = Optional::Some(3);
        let none: Optional<i32> = Optional::None;
        assert_eq!(a.clone() + b.clone(), Optional::Some(5));
        assert_eq!(a.clone() + none.clone(), Optional::Some(2));
        assert_eq!(none.clone() + b.clone(), Optional::Some(3));
        assert_eq!(none.clone() + none.clone(), Optional::None);
    }

    #[test]
    fn test_sub() {
        let a = Optional::Some(5);
        let b = Optional::Some(3);
        let none: Optional<i32> = Optional::None;
        assert_eq!(a.clone() - b.clone(), Optional::Some(2));
        assert_eq!(a.clone() - none.clone(), Optional::Some(5));
        assert_eq!(none.clone() - b.clone(), Optional::Some(3));
        assert_eq!(none.clone() - none.clone(), Optional::None);
    }

    #[test]
    fn test_mul() {
        let a = Optional::Some(2);
        let b = Optional::Some(3);
        let none: Optional<i32> = Optional::None;
        assert_eq!(a.clone() * b.clone(), Optional::Some(6));
        assert_eq!(a.clone() * none.clone(), Optional::Some(2));
        assert_eq!(none.clone() * b.clone(), Optional::Some(3));
        assert_eq!(none.clone() * none.clone(), Optional::None);
    }

    #[test]
    fn test_div() {
        let a = Optional::Some(6);
        let b = Optional::Some(3);
        let none: Optional<i32> = Optional::None;
        assert_eq!(a.clone() / b.clone(), Optional::Some(2));
        assert_eq!(a.clone() / none.clone(), Optional::Some(6));
        assert_eq!(none.clone() / b.clone(), Optional::Some(3));
        assert_eq!(none.clone() / none.clone(), Optional::None);
    }
}
