use regex::Regex;

/// Trait representing a formula with name, body, and dependencies.
///
/// Implement this trait to create custom formula types that can be used with the [`crate::Engine`].
pub trait FormulaT {
    fn name(&self) -> &str;
    fn body(&self) -> &str;
    fn depends_on(&self) -> &[String];
}

/// A formula with a name, body, and automatically detected dependencies.
///
/// Dependencies are automatically extracted from `get_output_from('formula_name')` calls
/// in the formula body. The engine uses these dependencies to determine execution order.
///
/// # Examples
///
/// ```
/// use formcalc::Formula;
///
/// // Simple formula with no dependencies
/// let formula = Formula::new("simple", "return 2 + 2");
///
/// // Formula that depends on another formula
/// let dependent = Formula::new("result", "return get_output_from('simple') * 10");
/// ```
#[derive(Debug, Clone)]
pub struct Formula {
    name: String,
    body: String,
    depends_on: Vec<String>,
}

impl Formula {
    /// Creates a new formula with the given name and body.
    ///
    /// Dependencies are automatically extracted from `get_output_from('name')` calls
    /// in the formula body using regex pattern matching.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for this formula
    /// * `body` - The formula expression to evaluate
    ///
    /// # Examples
    ///
    /// ```
    /// use formcalc::Formula;
    ///
    /// let formula = Formula::new("tax", "return price * 0.2");
    /// let with_dep = Formula::new("total", "return get_output_from('tax') + get_output_from('price')");
    /// ```
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
