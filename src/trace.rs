use crate::value::Value;

/// Configures opt-in execution tracing.
///
/// Tracing is disabled by default. Each formula retains at most
/// `max_steps_per_formula` expression records, which bounds trace memory independently
/// of formula complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceOptions {
    pub max_steps_per_formula: usize,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            max_steps_per_formula: 1_000,
        }
    }
}

/// A completed expression evaluation within a formula.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceStep {
    pub index: usize,
    pub operation: &'static str,
    pub result: Value,
}

/// The evaluation outcome and bounded expression trace for one formula.
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaTrace {
    pub formula_name: String,
    pub dependencies: Vec<String>,
    pub result: std::result::Result<Value, String>,
    pub steps: Vec<TraceStep>,
    pub truncated: bool,
}

/// A deterministic, formula-name-ordered snapshot of an execution trace.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExecutionTrace {
    pub formulas: Vec<FormulaTrace>,
}
