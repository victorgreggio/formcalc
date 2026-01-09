#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Number(f64),
    String(String),
    Bool(bool),
    Identifier(String),

    // Binary operations
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Power(Box<Expr>, Box<Expr>),
    Modulo(Box<Expr>, Box<Expr>),

    // Comparison
    Equal(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    LessThanOrEqual(Box<Expr>, Box<Expr>),
    GreaterThanOrEqual(Box<Expr>, Box<Expr>),

    // Logical
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    // Unary
    UnaryMinus(Box<Expr>),

    // Function calls
    FunctionCall { name: String, args: Vec<Expr> },

    // Built-in functions
    Max(Box<Expr>, Box<Expr>),
    Min(Box<Expr>, Box<Expr>),
    Rnd(Box<Expr>, Box<Expr>),
    Ceil(Box<Expr>),
    Floor(Box<Expr>),
    Exp(Box<Expr>),
    Year(Box<Expr>),
    Month(Box<Expr>),
    Day(Box<Expr>),
    Substr(Box<Expr>, Box<Expr>, Box<Expr>),
    AddDays(Box<Expr>, Box<Expr>),
    GetDiffDays(Box<Expr>, Box<Expr>),
    PaddedString(Box<Expr>, Box<Expr>),
    DifferenceInMonths(Box<Expr>, Box<Expr>),
    GetOutputFrom(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Return(Expr),
    If {
        condition: Expr,
        then_block: Box<Statement>,
        else_ifs: Vec<(Expr, Statement)>,
        else_block: Option<Box<Statement>>,
    },
    Error(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statement: Statement,
}
