use crate::cache::{FormulaResultCache, FunctionCache, FunctionResultCache, VariableCache};
use crate::error::{CalculatorError, Result};
use crate::formula::{Formula, FormulaT};
use crate::function::{build_function_id, Function};
use crate::graph::DAGraph;
use crate::parser::{Evaluator, Parser};
use crate::value::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Main engine for executing formulas
pub struct Engine {
    variable_cache: VariableCache,
    formula_result_cache: FormulaResultCache,
    function_cache: FunctionCache,
    function_result_cache: FunctionResultCache,
    errors: HashMap<String, String>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            variable_cache: VariableCache::new(),
            formula_result_cache: FormulaResultCache::new(),
            function_cache: FunctionCache::new(),
            function_result_cache: FunctionResultCache::new(),
            errors: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variable_cache.set(name, value);
    }

    /// Register a custom function
    pub fn register_function(&mut self, function: Arc<dyn Function>) {
        let function_id = build_function_id(function.name(), function.num_args());
        self.function_cache.set(function_id, function);
    }

    /// Execute multiple formulas with dependency resolution
    pub fn execute(&mut self, formulas: Vec<Formula>) -> Result<()> {
        let mut graph = DAGraph::new();

        // Build dependency graph
        for formula in &formulas {
            graph
                .add_node(
                    formula.name().to_string(),
                    formula.clone(),
                    formula.depends_on().to_vec(),
                )
                .map_err(|e| CalculatorError::DependencyError(e))?;
        }

        // Topological sort to get execution order
        let (layers, detached) = graph.topological_sort();

        // Handle detached (unresolvable) formulas
        for formula_name in detached {
            let error_msg = format!("Could not resolve dependency path for formula: '{}'", formula_name);
            self.errors.insert(formula_name, error_msg);
        }

        // Execute formulas layer by layer
        for layer in layers {
            // In a real implementation, these could be executed in parallel
            for formula_name in layer {
                if let Some(formula) = graph.get(&formula_name) {
                    self.execute_formula(formula);
                }
            }
        }

        Ok(())
    }

    fn execute_formula(&mut self, formula: &Formula) {
        match self.try_execute_formula(formula) {
            Ok(result) => {
                self.formula_result_cache.set(formula.name().to_string(), result);
            }
            Err(e) => {
                let error_msg = format!("Error executing formula '{}': {}", formula.name(), e);
                self.errors.insert(formula.name().to_string(), error_msg);
            }
        }
    }

    fn try_execute_formula(&self, formula: &Formula) -> Result<Value> {
        let mut parser = Parser::new(formula.body())?;
        let program = parser.parse()?;

        let evaluator = Evaluator::new(
            self.variable_cache.clone(),
            self.formula_result_cache.clone(),
            self.function_cache.clone(),
            self.function_result_cache.clone(),
        );

        evaluator.evaluate(&program)
    }

    /// Get the result of a formula
    pub fn get_result(&self, formula_name: &str) -> Option<Value> {
        self.formula_result_cache.get(formula_name)
    }

    /// Get all errors that occurred during execution
    pub fn get_errors(&self) -> &HashMap<String, String> {
        &self.errors
    }

    /// Clear all caches and errors
    pub fn clear(&mut self) {
        self.variable_cache.clear();
        self.formula_result_cache.clear();
        self.function_result_cache.clear();
        self.errors.clear();
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_formula() {
        let mut engine = Engine::new();
        let formula = Formula::new("test", "return 2 + 2");
        
        engine.execute(vec![formula]).unwrap();
        
        let result = engine.get_result("test").unwrap();
        assert_eq!(result, Value::Number(4.0));
    }

    #[test]
    fn test_formula_with_variable() {
        let mut engine = Engine::new();
        engine.set_variable("x".to_string(), Value::Number(10.0));
        
        let formula = Formula::new("test", "return x * 2");
        engine.execute(vec![formula]).unwrap();
        
        let result = engine.get_result("test").unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_formula_dependencies() {
        let mut engine = Engine::new();
        
        let formula1 = Formula::new("first", "return 10");
        let formula2 = Formula::new("second", "return GetOutputFrom('first') * 2");
        
        engine.execute(vec![formula1, formula2]).unwrap();
        
        let result = engine.get_result("second").unwrap();
        assert_eq!(result, Value::Number(20.0));
    }

    #[test]
    fn test_if_statement() {
        let mut engine = Engine::new();
        let formula = Formula::new("test", "if (5 > 3) then return 100 else return 200 end");
        
        engine.execute(vec![formula]).unwrap();
        
        let result = engine.get_result("test").unwrap();
        assert_eq!(result, Value::Number(100.0));
    }
}
