use std::cmp::Ordering;
use std::fmt;

/// Represents a value that can be a string, number, or boolean.
///
/// This is the primary data type for all values in the formula engine,
/// including variables, function parameters, and formula results.
///
/// # Examples
///
/// ```
/// use formcalc::Value;
///
/// let num = Value::Number(42.0);
/// let text = Value::String("hello".to_string());
/// let flag = Value::Bool(true);
///
/// assert_eq!(num.as_number(), Some(42.0));
/// assert_eq!(text.as_string(), Some("hello"));
/// assert_eq!(flag.as_bool(), Some(true));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A string value
    String(String),
    /// A numeric value (f64)
    Number(f64),
    /// A boolean value
    Bool(bool),
}

impl Value {
    /// Returns `true` if the value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Returns `true` if the value is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Returns `true` if the value is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Returns the value as a string slice if it is a string, or `None` otherwise.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as an f64 if it is a number, or `None` otherwise.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the value as a boolean if it is a boolean, or `None` otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the underlying value as an object representation
    pub fn get(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        let num = Value::from(42.0);
        assert!(num.is_number());
        assert_eq!(num.as_number(), Some(42.0));

        let text = Value::from("hello");
        assert!(text.is_string());
        assert_eq!(text.as_string(), Some("hello"));

        let flag = Value::from(true);
        assert!(flag.is_bool());
        assert_eq!(flag.as_bool(), Some(true));
    }

    #[test]
    fn test_value_comparison() {
        let a = Value::from(5.0);
        let b = Value::from(10.0);
        assert!(a < b);

        let x = Value::from("apple");
        let y = Value::from("banana");
        assert!(x < y);
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::from(42.5).to_string(), "42.5");
        assert_eq!(Value::from("test").to_string(), "test");
        assert_eq!(Value::from(true).to_string(), "true");
    }
}
