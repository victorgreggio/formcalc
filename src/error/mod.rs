use thiserror::Error;

/// Errors that can occur during formula parsing and evaluation.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CalculatorError {
    #[error("Evaluation error: {0}")]
    EvalError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Error function called: {0}")]
    ErrorCall(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Formula not found: {0}")]
    FormulaNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Dependency error: {0}")]
    DependencyError(String),

    #[error("Date parsing error: {0}")]
    DateParseError(String),

    #[error("Division by zero")]
    DivisionByZero,
}

/// A specialized `Result` type for formula operations.
///
/// This is a convenience alias for `Result<T, CalculatorError>`.
pub type Result<T> = std::result::Result<T, CalculatorError>;
