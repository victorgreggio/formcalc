use super::ast::{Expr, Program, Statement};
use crate::cache::{FormulaResultCache, FunctionCache, FunctionResultCache, VariableCache};
use crate::error::{CalculatorError, Result};
use crate::function::build_function_id;
use crate::value::Value;
use chrono::{Datelike, NaiveDateTime};

pub struct Evaluator {
    variable_cache: VariableCache,
    formula_result_cache: FormulaResultCache,
    function_cache: FunctionCache,
    function_result_cache: FunctionResultCache,
}

impl Evaluator {
    pub fn new(
        variable_cache: VariableCache,
        formula_result_cache: FormulaResultCache,
        function_cache: FunctionCache,
        function_result_cache: FunctionResultCache,
    ) -> Self {
        Self {
            variable_cache,
            formula_result_cache,
            function_cache,
            function_result_cache,
        }
    }

    pub fn evaluate(&self, program: &Program) -> Result<Value> {
        self.evaluate_statement(&program.statement)
    }

    fn evaluate_statement(&self, stmt: &Statement) -> Result<Value> {
        match stmt {
            Statement::Return(expr) => self.evaluate_expr(expr),
            Statement::If {
                condition,
                then_block,
                else_ifs,
                else_block,
            } => {
                let cond_val = self.evaluate_expr(condition)?;
                let cond_bool = cond_val.as_bool().ok_or_else(|| {
                    CalculatorError::TypeError("Condition must be boolean".to_string())
                })?;

                if cond_bool {
                    return self.evaluate_statement(then_block);
                }

                for (else_if_cond, else_if_block) in else_ifs {
                    let else_if_val = self.evaluate_expr(else_if_cond)?;
                    let else_if_bool = else_if_val.as_bool().ok_or_else(|| {
                        CalculatorError::TypeError("Else-if condition must be boolean".to_string())
                    })?;

                    if else_if_bool {
                        return self.evaluate_statement(else_if_block);
                    }
                }

                if let Some(else_blk) = else_block {
                    self.evaluate_statement(else_blk)
                } else {
                    Err(CalculatorError::EvalError(
                        "No matching condition".to_string(),
                    ))
                }
            }
            Statement::Error(expr) => {
                let val = self.evaluate_expr(expr)?;
                let msg = match val {
                    Value::String(s) => format!("Error function called with message: {}", s),
                    Value::Number(n) => format!("Error function called with code: {}", n),
                    Value::Bool(b) => format!("Error function called with value: {}", b),
                };
                Err(CalculatorError::ErrorCall(msg))
            }
        }
    }

    fn evaluate_expr(&self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Identifier(name) => self
                .variable_cache
                .get(name)
                .ok_or_else(|| CalculatorError::VariableNotFound(name.clone())),

            // Arithmetic
            Expr::Add(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (&l, &r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    _ => Ok(Value::String(format!("{}{}", l.get(), r.get()))),
                }
            }
            Expr::Subtract(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    _ => Err(CalculatorError::TypeError(
                        "Subtraction requires numbers".to_string(),
                    )),
                }
            }
            Expr::Multiply(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    _ => Err(CalculatorError::TypeError(
                        "Multiplication requires numbers".to_string(),
                    )),
                }
            }
            Expr::Divide(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => {
                        if b == 0.0 {
                            Err(CalculatorError::DivisionByZero)
                        } else {
                            Ok(Value::Number(a / b))
                        }
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Division requires numbers".to_string(),
                    )),
                }
            }
            Expr::Power(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(b))),
                    _ => Err(CalculatorError::TypeError(
                        "Power requires numbers".to_string(),
                    )),
                }
            }
            Expr::Modulo(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a % b)),
                    _ => Err(CalculatorError::TypeError(
                        "Modulo requires numbers".to_string(),
                    )),
                }
            }

            // Comparison
            Expr::Equal(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;
                Ok(Value::Bool(l == r))
            }
            Expr::NotEqual(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;
                Ok(Value::Bool(l != r))
            }
            Expr::LessThan(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match l.partial_cmp(&r) {
                    Some(ord) => Ok(Value::Bool(ord == std::cmp::Ordering::Less)),
                    None => Err(CalculatorError::TypeError(
                        "Cannot compare values of different types".to_string(),
                    )),
                }
            }
            Expr::GreaterThan(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match l.partial_cmp(&r) {
                    Some(ord) => Ok(Value::Bool(ord == std::cmp::Ordering::Greater)),
                    None => Err(CalculatorError::TypeError(
                        "Cannot compare values of different types".to_string(),
                    )),
                }
            }
            Expr::LessThanOrEqual(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match l.partial_cmp(&r) {
                    Some(ord) => Ok(Value::Bool(ord != std::cmp::Ordering::Greater)),
                    None => Err(CalculatorError::TypeError(
                        "Cannot compare values of different types".to_string(),
                    )),
                }
            }
            Expr::GreaterThanOrEqual(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match l.partial_cmp(&r) {
                    Some(ord) => Ok(Value::Bool(ord != std::cmp::Ordering::Less)),
                    None => Err(CalculatorError::TypeError(
                        "Cannot compare values of different types".to_string(),
                    )),
                }
            }

            // Logical
            Expr::And(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    _ => Err(CalculatorError::TypeError(
                        "Logical AND requires booleans".to_string(),
                    )),
                }
            }
            Expr::Or(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Err(CalculatorError::TypeError(
                        "Logical OR requires booleans".to_string(),
                    )),
                }
            }
            Expr::Not(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(CalculatorError::TypeError(
                        "Logical NOT requires boolean".to_string(),
                    )),
                }
            }

            // Unary
            Expr::UnaryMinus(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(CalculatorError::TypeError(
                        "Unary minus requires number".to_string(),
                    )),
                }
            }

            // Built-in functions
            Expr::Max(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.max(b))),
                    _ => Err(CalculatorError::TypeError(
                        "Max requires numbers".to_string(),
                    )),
                }
            }
            Expr::Min(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.min(b))),
                    _ => Err(CalculatorError::TypeError(
                        "Min requires numbers".to_string(),
                    )),
                }
            }
            Expr::Rnd(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(value), Value::Number(decimals)) => {
                        let factor = 10_f64.powi(decimals as i32);
                        Ok(Value::Number((value * factor).round() / factor))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Rnd requires numbers".to_string(),
                    )),
                }
            }
            Expr::Ceil(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::Number(n) => Ok(Value::Number(n.ceil())),
                    _ => Err(CalculatorError::TypeError(
                        "Ceil requires number".to_string(),
                    )),
                }
            }
            Expr::Floor(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::Number(n) => Ok(Value::Number(n.floor())),
                    _ => Err(CalculatorError::TypeError(
                        "Floor requires number".to_string(),
                    )),
                }
            }
            Expr::Exp(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::Number(n) => Ok(Value::Number(n.exp())),
                    _ => Err(CalculatorError::TypeError(
                        "Exp requires number".to_string(),
                    )),
                }
            }
            Expr::Year(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::String(s) => {
                        let date = parse_date(&s)?;
                        Ok(Value::Number(date.year() as f64))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Year requires string date".to_string(),
                    )),
                }
            }
            Expr::Month(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::String(s) => {
                        let date = parse_date(&s)?;
                        Ok(Value::Number(date.month() as f64))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Month requires string date".to_string(),
                    )),
                }
            }
            Expr::Day(expr) => {
                let val = self.evaluate_expr(expr)?;

                match val {
                    Value::String(s) => {
                        let date = parse_date(&s)?;
                        Ok(Value::Number(date.day() as f64))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Day requires string date".to_string(),
                    )),
                }
            }
            Expr::Substr(str_expr, start_expr, len_expr) => {
                let s = self.evaluate_expr(str_expr)?;
                let start = self.evaluate_expr(start_expr)?;
                let len = self.evaluate_expr(len_expr)?;

                match (s, start, len) {
                    (Value::String(s), Value::Number(start), Value::Number(len)) => {
                        let start = start as usize;
                        let len = len as usize;
                        let result = s.chars().skip(start).take(len).collect::<String>();
                        Ok(Value::String(result))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Substr requires (string, number, number)".to_string(),
                    )),
                }
            }
            Expr::AddDays(date_expr, days_expr) => {
                let date_val = self.evaluate_expr(date_expr)?;
                let days_val = self.evaluate_expr(days_expr)?;

                match (date_val, days_val) {
                    (Value::String(s), Value::Number(days)) => {
                        let date = parse_date(&s)?;
                        let new_date = date + chrono::Duration::days(days as i64);
                        Ok(Value::String(
                            new_date.format("%Y-%m-%dT%H:%M:%S").to_string(),
                        ))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "AddDays requires (string date, number)".to_string(),
                    )),
                }
            }
            Expr::GetDiffDays(date1_expr, date2_expr) => {
                let date1_val = self.evaluate_expr(date1_expr)?;
                let date2_val = self.evaluate_expr(date2_expr)?;

                match (date1_val, date2_val) {
                    (Value::String(s1), Value::String(s2)) => {
                        let date1 = parse_date(&s1)?;
                        let date2 = parse_date(&s2)?;
                        let diff = (date1 - date2).num_days();
                        Ok(Value::Number(diff as f64))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "GetDiffDays requires two string dates".to_string(),
                    )),
                }
            }
            Expr::PaddedString(str_expr, width_expr) => {
                let s = self.evaluate_expr(str_expr)?;
                let width = self.evaluate_expr(width_expr)?;

                match (s, width) {
                    (Value::String(s), Value::Number(width)) => {
                        let width = width as usize;
                        let padded = format!("{:0>width$}", s);
                        Ok(Value::String(padded))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "PaddedString requires (string, number)".to_string(),
                    )),
                }
            }
            Expr::GetDiffMonths(date1_expr, date2_expr) => {
                let date1_val = self.evaluate_expr(date1_expr)?;
                let date2_val = self.evaluate_expr(date2_expr)?;

                match (date1_val, date2_val) {
                    (Value::String(s1), Value::String(s2)) => {
                        let date1 = parse_date(&s1)?;
                        let date2 = parse_date(&s2)?;
                        let months = (date1.year() - date2.year()) * 12
                            + (date1.month() as i32 - date2.month() as i32);
                        Ok(Value::Number(months.abs() as f64))
                    }
                    _ => Err(CalculatorError::TypeError(
                        "GetDiffMonths requires two string dates".to_string(),
                    )),
                }
            }
            Expr::GetOutputFrom(formula_expr) => {
                let formula_name = self.evaluate_expr(formula_expr)?;

                match formula_name {
                    Value::String(name) => self
                        .formula_result_cache
                        .get(&name)
                        .ok_or(CalculatorError::FormulaNotFound(name)),
                    _ => Err(CalculatorError::TypeError(
                        "GetOutputFrom requires string".to_string(),
                    )),
                }
            }

            // Custom function calls
            Expr::FunctionCall { name, args } => {
                let function_id = build_function_id(name, args.len());

                // Check cache first
                if let Some(cached) = self.function_result_cache.get(&function_id) {
                    return Ok(cached);
                }

                let function = self
                    .function_cache
                    .get(&function_id)
                    .ok_or_else(|| CalculatorError::FunctionNotFound(function_id.clone()))?;

                let mut param_values = Vec::new();
                for arg in args {
                    param_values.push(self.evaluate_expr(arg)?);
                }

                let result = function.execute(&param_values)?;
                self.function_result_cache.set(function_id, result.clone());
                Ok(result)
            }
        }
    }
}

fn parse_date(s: &str) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        })
        .map_err(|e| {
            CalculatorError::DateParseError(format!("Failed to parse date '{}': {}", s, e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::Parser;

    fn create_evaluator() -> Evaluator {
        Evaluator::new(
            VariableCache::new(),
            FormulaResultCache::new(),
            FunctionCache::new(),
            FunctionResultCache::new(),
        )
    }

    #[test]
    fn test_evaluate_number() {
        let mut parser = Parser::new("return 42").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_evaluate_addition() {
        let mut parser = Parser::new("return 2 + 3").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_evaluate_if_true() {
        let mut parser = Parser::new("if (5 > 3) then return 100 else return 200 end").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_evaluate_if_false() {
        let mut parser = Parser::new("if (3 > 5) then return 100 else return 200 end").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(200.0));
    }
}
