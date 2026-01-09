use crate::error::Result;
use crate::value::Value;

/// Trait for custom functions that can be called from formulas
pub trait Function: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    
    /// Get the number of arguments this function expects
    fn num_args(&self) -> usize;
    
    /// Execute the function with the given parameters
    fn execute(&self, params: &[Value]) -> Result<Value>;
}

/// Build a function ID from name and number of arguments
/// This matches the C# FunctionIdBuilder.Create implementation
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
