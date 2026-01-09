use crate::function::Function;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Cache for storing variables
#[derive(Debug, Clone, Default)]
pub struct VariableCache {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl VariableCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: Value) {
        self.cache.write().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

/// Cache for storing formula results
#[derive(Debug, Clone, Default)]
pub struct FormulaResultCache {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl FormulaResultCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, formula_name: String, value: Value) {
        self.cache.write().unwrap().insert(formula_name, value);
    }

    pub fn get(&self, formula_name: &str) -> Option<Value> {
        self.cache.read().unwrap().get(formula_name).cloned()
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

/// Cache for storing functions by their ID (name_numargs)
#[derive(Clone, Default)]
pub struct FunctionCache {
    cache: Arc<RwLock<HashMap<String, Arc<dyn Function>>>>,
}

impl FunctionCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, function_id: String, function: Arc<dyn Function>) {
        self.cache.write().unwrap().insert(function_id, function);
    }

    pub fn get(&self, function_id: &str) -> Option<Arc<dyn Function>> {
        self.cache.read().unwrap().get(function_id).cloned()
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

/// Cache for storing function results
#[derive(Debug, Clone, Default)]
pub struct FunctionResultCache {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl FunctionResultCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: Value) {
        self.cache.write().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_cache() {
        let cache = VariableCache::new();
        cache.set("x".to_string(), Value::from(42.0));

        assert_eq!(cache.get("x"), Some(Value::from(42.0)));
        assert_eq!(cache.get("y"), None);

        cache.clear();
        assert_eq!(cache.get("x"), None);
    }

    #[test]
    fn test_formula_result_cache() {
        let cache = FormulaResultCache::new();
        cache.set("formula1".to_string(), Value::from("result"));

        assert_eq!(cache.get("formula1"), Some(Value::from("result")));
        assert_eq!(cache.get("formula2"), None);
    }
}
