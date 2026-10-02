/// Trait representing a formula with a name and source body.
///
/// Implement this trait to create custom formula types that can be used with the [`crate::Engine`].
pub trait FormulaT {
    fn name(&self) -> &str;
    fn body(&self) -> &str;
}

/// A formula with a name and source body.
///
/// Dependencies are extracted from the parsed formula when the engine compiles it. Formula
/// construction therefore remains allocation-light and cannot disagree with the language parser.
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
}

impl Formula {
    /// Creates a new formula with the given name and body.
    ///
    /// Dependencies are extracted by [`crate::Engine`] from the parsed formula body.
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

        Self { name, body }
    }
}

impl FormulaT for Formula {
    fn name(&self) -> &str {
        &self.name
    }

    fn body(&self) -> &str {
        &self.body
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
    }
}
