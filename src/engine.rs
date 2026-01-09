use crate::cache::{FormulaResultCache, FunctionCache, FunctionResultCache, VariableCache};
use crate::error::{CalculatorError, Result};
use crate::formula::{Formula, FormulaT};
use crate::function::{build_function_id, Function};
use crate::graph::DAGraph;
use crate::parser::{Evaluator, Parser};
use crate::value::Value;
use rayon::prelude::*;
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
                .map_err(CalculatorError::DependencyError)?;
        }

        // Topological sort to get execution order
        let (layers, detached) = graph.topological_sort();

        // Handle detached (unresolvable) formulas
        for formula_name in detached {
            let error_msg = format!(
                "Could not resolve dependency path for formula: '{}'",
                formula_name
            );
            self.errors.insert(formula_name, error_msg);
        }

        // Execute formulas layer by layer
        // Formulas in the same layer can be executed in parallel
        for layer in layers {
            self.execute_layer_parallel(&graph, layer);
        }

        Ok(())
    }

    /// Execute all formulas in a layer in parallel
    fn execute_layer_parallel(&mut self, graph: &DAGraph<String, Formula>, layer: Vec<String>) {
        // Execute formulas in parallel
        let results: Vec<(String, Result<Value>)> = layer
            .par_iter()
            .filter_map(|formula_name| {
                graph.get(formula_name).map(|formula| {
                    let result = self.try_execute_formula(formula);
                    (formula_name.clone(), result)
                })
            })
            .collect();

        // Process results sequentially to update caches and collect errors
        for (formula_name, result) in results {
            match result {
                Ok(value) => {
                    self.formula_result_cache.set(formula_name, value);
                }
                Err(e) => {
                    let error_msg = format!("Error executing formula '{}': {}", formula_name, e);
                    self.errors.insert(formula_name, error_msg);
                }
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
        let formula2 = Formula::new("second", "return get_output_from('first') * 2");

        engine.execute(vec![formula1, formula2]).unwrap();

        // Check for errors
        if !engine.get_errors().is_empty() {
            for (name, error) in engine.get_errors() {
                eprintln!("Error in {}: {}", name, error);
            }
        }

        let result = engine
            .get_result("second")
            .expect("second formula should have result");
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

    #[test]
    fn test_parallel_execution() {
        let mut engine = Engine::new();

        // Create multiple independent formulas that can be executed in parallel
        let formulas = vec![
            Formula::new("a", "return 1 + 1"),
            Formula::new("b", "return 2 + 2"),
            Formula::new("c", "return 3 + 3"),
            Formula::new("d", "return 4 + 4"),
            Formula::new("e", "return 5 + 5"),
        ];

        engine.execute(formulas).unwrap();

        assert_eq!(engine.get_result("a").unwrap(), Value::Number(2.0));
        assert_eq!(engine.get_result("b").unwrap(), Value::Number(4.0));
        assert_eq!(engine.get_result("c").unwrap(), Value::Number(6.0));
        assert_eq!(engine.get_result("d").unwrap(), Value::Number(8.0));
        assert_eq!(engine.get_result("e").unwrap(), Value::Number(10.0));
    }

    #[test]
    fn test_parallel_with_dependencies() {
        let mut engine = Engine::new();

        // Layer 0: a, b (can execute in parallel)
        // Layer 1: c, d (can execute in parallel, both depend on layer 0)
        // Layer 2: e (depends on layer 1)
        let formulas = vec![
            Formula::new("a", "return 10"),
            Formula::new("b", "return 20"),
            Formula::new("c", "return get_output_from('a') * 2"),
            Formula::new("d", "return get_output_from('b') * 2"),
            Formula::new("e", "return get_output_from('c') + get_output_from('d')"),
        ];

        engine.execute(formulas).unwrap();

        assert_eq!(engine.get_result("a").unwrap(), Value::Number(10.0));
        assert_eq!(engine.get_result("b").unwrap(), Value::Number(20.0));
        assert_eq!(engine.get_result("c").unwrap(), Value::Number(20.0));
        assert_eq!(engine.get_result("d").unwrap(), Value::Number(40.0));
        assert_eq!(engine.get_result("e").unwrap(), Value::Number(60.0));
    }
}
