use regex::Regex;

/// Trait representing a formula with name, body, and dependencies
pub trait FormulaT {
    fn name(&self) -> &str;
    fn body(&self) -> &str;
    fn depends_on(&self) -> &[String];
}

/// Concrete implementation of a formula
#[derive(Debug, Clone)]
pub struct Formula {
    name: String,
    body: String,
    depends_on: Vec<String>,
}

impl Formula {
    /// Create a new formula with the given name and body
    pub fn new(name: impl Into<String>, body: impl Into<String>) -> Self {
        let name = name.into();
        let body = body.into();
        let depends_on = Self::build_depends_on(&body);
        
        Self {
            name,
            body,
            depends_on,
        }
    }

    /// Extract dependencies from the formula body by finding get_output_from calls
    /// Pattern: get_output_from('formula_name')
    fn build_depends_on(body: &str) -> Vec<String> {
        // Rust regex doesn't support lookahead/lookbehind, so we'll use a simpler approach
        let pattern = r"get_output_from\('([^']+)'\)";
        let re = Regex::new(pattern).unwrap();
        
        re.captures_iter(body)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}

impl FormulaT for Formula {
    fn name(&self) -> &str {
        &self.name
    }

    fn body(&self) -> &str {
        &self.body
    }

    fn depends_on(&self) -> &[String] {
        &self.depends_on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formula_creation() {
        let formula = Formula::new("test", "return 1 + 1");
        assert_eq!(formula.name(), "test");
        assert_eq!(formula.body(), "return 1 + 1");
        assert_eq!(formula.depends_on().len(), 0);
    }

    #[test]
    fn test_formula_dependencies() {
        let body = "return get_output_from('formula1') + get_output_from('formula2')";
        let formula = Formula::new("test", body);
        
        assert_eq!(formula.depends_on().len(), 2);
        assert!(formula.depends_on().contains(&"formula1".to_string()));
        assert!(formula.depends_on().contains(&"formula2".to_string()));
    }

    #[test]
    fn test_formula_no_dependencies() {
        let formula = Formula::new("simple", "return 42");
        assert_eq!(formula.depends_on().len(), 0);
    }
}
