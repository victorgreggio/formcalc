//! # FormCalc - Formula Calculator Engine
//!
//! A powerful and flexible formula evaluation engine with automatic dependency resolution
//! and parallel execution capabilities.
//!
//! ## Features
//!
//! - **Formula Parsing**: Parse and evaluate complex formulas with arithmetic, logical, and comparison operations
//! - **Dependency Management**: Automatically resolve and execute formulas in the correct order based on dependencies
//! - **Parallel Execution**: Formulas in the same dependency layer are executed in parallel for maximum performance
//! - **Built-in Functions**: Support for mathematical, string, and date functions
//! - **Custom Functions**: Register custom functions to extend functionality
//! - **Variables**: Support for variables in formulas
//! - **Type System**: Strong typing with support for numbers, strings, and booleans
//! - **Error Handling**: Comprehensive error reporting with detailed messages
//!
//! ## Quick Start
//!
//! ```rust
//! use formcalc::{Engine, Formula, Value};
//!
//! let mut engine = Engine::new();
//!
//! // Simple calculation
//! let formula = Formula::new("calculation", "return 2 + 2 * 3");
//! engine.execute(vec![formula]).unwrap();
//! let result = engine.get_result("calculation").unwrap();
//! assert_eq!(result, Value::Number(8.0));
//! ```
//!
//! ## Using Variables
//!
//! ```rust
//! use formcalc::{Engine, Formula, Value};
//!
//! let mut engine = Engine::new();
//! engine.set_variable("price".to_string(), Value::Number(100.0));
//! engine.set_variable("tax_rate".to_string(), Value::Number(0.2));
//!
//! let formula = Formula::new("total", "return price * (1 + tax_rate)");
//! engine.execute(vec![formula]).unwrap();
//!
//! let result = engine.get_result("total").unwrap();
//! assert_eq!(result, Value::Number(120.0));
//! ```
//!
//! ## Formula Dependencies
//!
//! The engine automatically resolves dependencies between formulas:
//!
//! ```rust
//! use formcalc::{Engine, Formula, Value};
//!
//! let mut engine = Engine::new();
//!
//! let formula1 = Formula::new("base_price", "return 100");
//! let formula2 = Formula::new("with_tax", "return get_output_from('base_price') * 1.2");
//! let formula3 = Formula::new("final_price", "return get_output_from('with_tax') + 10");
//!
//! // The engine automatically resolves dependencies and executes in correct order
//! engine.execute(vec![formula1, formula2, formula3]).unwrap();
//!
//! let result = engine.get_result("final_price").unwrap();
//! assert_eq!(result, Value::Number(130.0));
//! ```
//!
//! ## Custom Functions
//!
//! Extend the engine with custom functions:
//!
//! ```rust
//! use formcalc::{Engine, Formula, Function, Value, Result, CalculatorError};
//! use std::sync::Arc;
//!
//! struct DoubleFunction;
//!
//! impl Function for DoubleFunction {
//!     fn name(&self) -> &str {
//!         "double"
//!     }
//!
//!     fn num_args(&self) -> usize {
//!         1
//!     }
//!
//!     fn execute(&self, params: &[Value]) -> Result<Value> {
//!         match params[0] {
//!             Value::Number(n) => Ok(Value::Number(n * 2.0)),
//!             _ => Err(CalculatorError::TypeError("Expected number".to_string())),
//!         }
//!     }
//! }
//!
//! let mut engine = Engine::new();
//! engine.register_function(Arc::new(DoubleFunction));
//!
//! let formula = Formula::new("test", "return double(21)");
//! engine.execute(vec![formula]).unwrap();
//!
//! let result = engine.get_result("test").unwrap();
//! assert_eq!(result, Value::Number(42.0));
//! ```

pub mod cache;
pub mod engine;
pub mod error;
pub mod formula;
pub mod function;
pub mod graph;
pub mod parser;
pub mod value;

// Re-export main types
pub use engine::Engine;
pub use error::{CalculatorError, Result};
pub use formula::{Formula, FormulaT};
pub use function::Function;
pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_calculation() {
        let mut engine = Engine::new();
        let formula = Formula::new("simple", "return 1 + 1");

        engine.execute(vec![formula]).unwrap();

        let result = engine.get_result("simple").unwrap();
        assert_eq!(result, Value::Number(2.0));
    }

    #[test]
    fn test_complex_expression() {
        let mut engine = Engine::new();
        let formula = Formula::new("complex", "return (5 + 3) * 2 - 1");

        engine.execute(vec![formula]).unwrap();

        let result = engine.get_result("complex").unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_string_concatenation() {
        let mut engine = Engine::new();
        let formula = Formula::new("concat", "return 'Hello' + ' ' + 'World'");

        engine.execute(vec![formula]).unwrap();

        let result = engine.get_result("concat").unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_builtin_functions() {
        let mut engine = Engine::new();
        let formula = Formula::new("funcs", "return max(10, 20) + min(5, 3)");

        engine.execute(vec![formula]).unwrap();

        let result = engine.get_result("funcs").unwrap();
        assert_eq!(result, Value::Number(23.0));
    }
}
