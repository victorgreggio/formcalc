use crate::error::Result;
use crate::value::Value;

/// Trait for custom functions that can be called from formulas.
///
/// Implement this trait to create custom functions that can be registered
/// with the [`crate::Engine`] and called from formula expressions.
///
/// Functions are identified by their name and number of arguments (arity),
/// allowing you to have multiple functions with the same name but different arities.
///
/// # Examples
///
/// ```
/// use formcalc::{Function, Value, Result, CalculatorError};
///
/// struct AddFunction;
///
/// impl Function for AddFunction {
///     fn name(&self) -> &str {
///         "add"
///     }
///
///     fn num_args(&self) -> usize {
///         2
///     }
///
///     fn execute(&self, params: &[Value]) -> Result<Value> {
///         match (&params[0], &params[1]) {
///             (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
///             _ => Err(CalculatorError::TypeError("Expected numbers".to_string())),
///         }
///     }
/// }
/// ```
pub trait Function: Send + Sync {
    /// Returns the function name.
    ///
    /// This is the name that will be used to call the function in formulas.
    fn name(&self) -> &str;

    /// Returns the number of arguments this function expects.
    ///
    /// The engine will validate that the correct number of arguments
    /// are provided when the function is called.
    fn num_args(&self) -> usize;

    /// Executes the function with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - A slice of [`Value`] parameters. The length will match `num_args()`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value)` with the function result, or an error if the function fails.
    fn execute(&self, params: &[Value]) -> Result<Value>;
}

/// Builds a function identifier from name and number of arguments.
///
/// The function ID is used internally to uniquely identify functions,
/// allowing multiple functions with the same name but different arities.
///
/// The name is converted to snake_case and combined with the argument count.
///
/// # Examples
///
/// ```
/// use formcalc::function::build_function_id;
///
/// assert_eq!(build_function_id("MyFunction", 2), "my_function_2");
/// assert_eq!(build_function_id("max", 2), "max_2");
/// ```
pub fn build_function_id(name: &str, num_args: usize) -> String {
    format!("{}_{}", to_snake_case(name), num_args)
}

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase if not first char and previous was lowercase or next is lowercase
            if i > 0 {
                let prev_is_lower = chars[i - 1].is_lowercase();
                let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
                if prev_is_lower || next_is_lower {
                    result.push('_');
                }
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_id_builder() {
        assert_eq!(build_function_id("MyFunction", 2), "my_function_2");
        assert_eq!(build_function_id("simpleFunc", 0), "simple_func_0");
        assert_eq!(build_function_id("UPPER", 1), "upper_1");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("MyFunction"), "my_function");
        assert_eq!(to_snake_case("simpleFunc"), "simple_func");
        assert_eq!(to_snake_case("lowercase"), "lowercase");
        assert_eq!(to_snake_case("UPPER"), "upper");
    }
}
