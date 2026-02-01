use crate::{Engine as CoreEngine, Formula as CoreFormula, Value as CoreValue};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// WASM wrapper for the FormCalc Engine
#[wasm_bindgen]
pub struct Engine {
    inner: CoreEngine,
}

#[wasm_bindgen]
impl Engine {
    /// Create a new Engine instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine {
            inner: CoreEngine::new(),
        }
    }

    /// Evaluate a simple expression with variables
    /// Returns the result as a number
    #[wasm_bindgen(js_name = evaluateExpression)]
    pub fn evaluate_expression(
        &mut self,
        expression: &str,
        variables: JsValue,
    ) -> Result<f64, JsValue> {
        // Parse variables from JavaScript object
        let vars: HashMap<String, f64> = serde_wasm_bindgen::from_value(variables)
            .map_err(|e| JsValue::from_str(&format!("Invalid variables: {}", e)))?;

        // Set variables in engine
        for (key, value) in vars {
            self.inner.set_variable(key, CoreValue::Number(value));
        }

        // Create a temporary formula
        let formula = CoreFormula::new("_temp", &format!("return {}", expression));

        // Execute formula
        self.inner
            .execute(vec![formula])
            .map_err(|e| JsValue::from_str(&format!("Execution error: {}", e)))?;

        // Get result
        let result = self
            .inner
            .get_result("_temp")
            .ok_or_else(|| JsValue::from_str("No result found"))?;

        // Convert to number
        match result {
            CoreValue::Number(n) => Ok(n),
            _ => Err(JsValue::from_str("Result is not a number")),
        }
    }

    /// Validate an expression syntax
    #[wasm_bindgen(js_name = validateExpression)]
    pub fn validate_expression(&self, expression: &str) -> bool {
        // Try to create a formula - if it fails, syntax is invalid
        let formula = CoreFormula::new("_test", &format!("return {}", expression));

        // Create a temporary engine to test
        let mut test_engine = CoreEngine::new();
        test_engine.execute(vec![formula]).is_ok()
    }
}

/// Simple formula parser for WASM
#[wasm_bindgen]
pub struct Formula {
    name: String,
    expression: String,
}

#[wasm_bindgen]
impl Formula {
    /// Create a new formula
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, expression: &str) -> Formula {
        Formula {
            name: name.to_string(),
            expression: expression.to_string(),
        }
    }

    /// Parse and validate a formula expression
    #[wasm_bindgen(js_name = parse)]
    pub fn parse(expression: &str) -> Result<Formula, JsValue> {
        // Try to create a formula to validate syntax
        let formula = CoreFormula::new("_parse_test", &format!("return {}", expression));

        // Test if it's valid
        let mut test_engine = CoreEngine::new();
        test_engine
            .execute(vec![formula])
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        Ok(Formula {
            name: "_parsed".to_string(),
            expression: expression.to_string(),
        })
    }

    /// Get formula name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get formula expression
    #[wasm_bindgen(getter)]
    pub fn expression(&self) -> String {
        self.expression.clone()
    }
}
