//! RecipeCalculator Engine - A formula evaluation engine
//!
//! This is a Rust implementation of the RecipeCalculator Engine,
//! providing formula parsing, evaluation, and dependency management.

pub mod value;
pub mod error;
pub mod formula;
pub mod function;
pub mod cache;
pub mod graph;
pub mod parser;
pub mod engine;

// Re-export main types
pub use value::Value;
pub use error::{CalculatorError, Result};
pub use formula::{Formula, FormulaT};
pub use function::Function;
pub use engine::Engine;

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
