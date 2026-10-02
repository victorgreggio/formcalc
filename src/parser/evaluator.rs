use super::ast::{Expr, Program, Statement};
use crate::cache::{FormulaResultCache, FunctionCache, VariableCache};
use crate::error::{CalculatorError, Result};
use crate::function::build_function_id;
use crate::trace::TraceStep;
use crate::value::Value;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};
use std::cell::RefCell;

pub struct Evaluator {
    variable_cache: VariableCache,
    formula_result_cache: FormulaResultCache,
    function_cache: FunctionCache,
    trace: RefCell<Option<TraceCollector>>,
}

#[derive(Debug)]
struct TraceCollector {
    max_steps: usize,
    steps: Vec<TraceStep>,
    truncated: bool,
}

impl Evaluator {
    pub fn new(
        variable_cache: VariableCache,
        formula_result_cache: FormulaResultCache,
        function_cache: FunctionCache,
    ) -> Self {
        Self {
            variable_cache,
            formula_result_cache,
            function_cache,
            trace: RefCell::new(None),
        }
    }

    pub fn evaluate(&self, program: &Program) -> Result<Value> {
        self.trace.replace(None);
        self.evaluate_statement(&program.statement)
    }

    pub fn evaluate_with_trace(
        &self,
        program: &Program,
        max_steps: usize,
    ) -> (Result<Value>, Vec<TraceStep>, bool) {
        self.trace.replace(Some(TraceCollector {
            max_steps,
            steps: Vec::with_capacity(max_steps),
            truncated: false,
        }));
        let result = self.evaluate_statement(&program.statement);
        let trace = self.trace.replace(None).expect("trace collector was set");
        (result, trace.steps, trace.truncated)
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
        let result = self.evaluate_expr_inner(expr);
        if let Ok(value) = &result {
            self.record_trace(expr, value);
        }
        result
    }

    fn evaluate_expr_inner(&self, expr: &Expr) -> Result<Value> {
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
                    (Value::Number(a), Value::Number(b)) => {
                        checked_decimal(a.checked_add(*b), "addition")
                    }
                    _ => Ok(Value::String(format!("{}{}", l.get(), r.get()))),
                }
            }
            Expr::Subtract(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => {
                        checked_decimal(a.checked_sub(b), "subtraction")
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Subtraction requires numbers".to_string(),
                    )),
                }
            }
            Expr::Multiply(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => {
                        checked_decimal(a.checked_mul(b), "multiplication")
                    }
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
                        if b.is_zero() {
                            Err(CalculatorError::DivisionByZero)
                        } else {
                            checked_decimal(a.checked_div(b), "division")
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
                    (Value::Number(a), Value::Number(b)) => {
                        if (a.is_zero() && b.is_sign_negative())
                            || (a.is_sign_negative() && !b.fract().is_zero())
                        {
                            Err(CalculatorError::EvalError(
                                "Power is undefined for these operands".to_string(),
                            ))
                        } else {
                            checked_decimal(a.checked_powd(b), "power")
                        }
                    }
                    _ => Err(CalculatorError::TypeError(
                        "Power requires numbers".to_string(),
                    )),
                }
            }
            Expr::Modulo(left, right) => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;

                match (l, r) {
                    (Value::Number(a), Value::Number(b)) => {
                        if b.is_zero() {
                            Err(CalculatorError::DivisionByZero)
                        } else {
                            checked_decimal(a.checked_rem(b), "modulo")
                        }
                    }
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
                match l {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    Value::Bool(true) => match self.evaluate_expr(right)? {
                        Value::Bool(value) => Ok(Value::Bool(value)),
                        _ => Err(CalculatorError::TypeError(
                            "Logical AND requires booleans".to_string(),
                        )),
                    },
                    _ => Err(CalculatorError::TypeError(
                        "Logical AND requires booleans".to_string(),
                    )),
                }
            }
            Expr::Or(left, right) => {
                let l = self.evaluate_expr(left)?;
                match l {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    Value::Bool(false) => match self.evaluate_expr(right)? {
                        Value::Bool(value) => Ok(Value::Bool(value)),
                        _ => Err(CalculatorError::TypeError(
                            "Logical OR requires booleans".to_string(),
                        )),
                    },
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
                        Ok(Value::Number(value.round_dp_with_strategy(
                            decimal_to_decimal_places(decimals, "Rnd decimal places")?,
                            RoundingStrategy::MidpointAwayFromZero,
                        )))
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
                    Value::Number(n) => checked_decimal(n.checked_exp(), "exponential"),
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
                        Ok(Value::Number(Decimal::from(date.year())))
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
                        Ok(Value::Number(Decimal::from(date.month())))
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
                        Ok(Value::Number(Decimal::from(date.day())))
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
                        let start = decimal_to_usize(start, "Substr start")?;
                        let len = decimal_to_usize(len, "Substr length")?;
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
                        let days = decimal_to_i64(days, "AddDays day count")?;
                        let duration = chrono::Duration::try_days(days).ok_or_else(|| {
                            CalculatorError::InvalidArgument(
                                "AddDays day count is outside the supported range".to_string(),
                            )
                        })?;
                        let new_date = date.checked_add_signed(duration).ok_or_else(|| {
                            CalculatorError::EvalError(
                                "AddDays result is outside the supported date range".to_string(),
                            )
                        })?;
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
                        Ok(Value::Number(Decimal::from(diff)))
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
                        let width = decimal_to_usize(width, "PaddedString width")?;
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
                        Ok(Value::Number(Decimal::from(months.abs())))
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
                if is_core_builtin(name) {
                    return self.evaluate_core_builtin(name, args);
                }

                let function_id = build_function_id(name, args.len());

                let function = self
                    .function_cache
                    .get(&function_id)
                    .ok_or_else(|| CalculatorError::FunctionNotFound(function_id.clone()))?;

                if !function.is_deterministic() {
                    return Err(CalculatorError::EvalError(format!(
                        "Function '{}' is not permitted in deterministic execution",
                        name
                    )));
                }

                let mut param_values = Vec::new();
                for arg in args {
                    param_values.push(self.evaluate_expr(arg)?);
                }

                function.execute(&param_values)
            }
        }
    }

    fn record_trace(&self, expr: &Expr, result: &Value) {
        let mut trace = self.trace.borrow_mut();
        let Some(trace) = trace.as_mut() else {
            return;
        };

        if trace.steps.len() == trace.max_steps {
            trace.truncated = true;
            return;
        }

        trace.steps.push(TraceStep {
            index: trace.steps.len(),
            operation: expression_operation(expr),
            result: result.clone(),
        });
    }

    fn evaluate_core_builtin(&self, name: &str, args: &[Expr]) -> Result<Value> {
        if name == "if" {
            require_arity(name, args, 3)?;
            return match self.evaluate_expr(&args[0])? {
                Value::Bool(true) => self.evaluate_expr(&args[1]),
                Value::Bool(false) => self.evaluate_expr(&args[2]),
                _ => Err(CalculatorError::TypeError(
                    "IF condition must be boolean".to_string(),
                )),
            };
        }

        let values = args
            .iter()
            .map(|arg| self.evaluate_expr(arg))
            .collect::<Result<Vec<_>>>()?;

        match name {
            "abs" => {
                require_arity(name, args, 1)?;
                Ok(Value::Number(number(&values[0], "ABS")?.abs()))
            }
            "sign" => {
                require_arity(name, args, 1)?;
                let value = number(&values[0], "SIGN")?;
                Ok(Value::Number(if value.is_zero() {
                    Decimal::ZERO
                } else if value.is_sign_negative() {
                    -Decimal::ONE
                } else {
                    Decimal::ONE
                }))
            }
            "roundup" | "round_up" => {
                require_arity(name, args, 2)?;
                let value = number(&values[0], "ROUNDUP")?;
                let places = decimal_to_decimal_places(
                    number(&values[1], "ROUNDUP decimals")?,
                    "ROUNDUP decimals",
                )?;
                let strategy = if value.is_sign_negative() {
                    RoundingStrategy::ToNegativeInfinity
                } else {
                    RoundingStrategy::ToPositiveInfinity
                };
                Ok(Value::Number(
                    value.round_dp_with_strategy(places, strategy),
                ))
            }
            "rounddown" | "round_down" | "trunc" => {
                require_arity(name, args, 2)?;
                let value = number(&values[0], "TRUNC")?;
                let places = decimal_to_decimal_places(
                    number(&values[1], "TRUNC decimals")?,
                    "TRUNC decimals",
                )?;
                Ok(Value::Number(
                    value.round_dp_with_strategy(places, RoundingStrategy::ToZero),
                ))
            }
            "clamp" => {
                require_arity(name, args, 3)?;
                let value = number(&values[0], "CLAMP")?;
                let min = number(&values[1], "CLAMP")?;
                let max = number(&values[2], "CLAMP")?;
                if min > max {
                    return Err(CalculatorError::InvalidArgument(
                        "CLAMP minimum must not exceed maximum".to_string(),
                    ));
                }
                Ok(Value::Number(value.clamp(min, max)))
            }
            "sum" | "average" | "min" | "max" => {
                require_at_least(name, args, 1)?;
                let numbers = numeric_values(&values, name)?;
                let result = match name {
                    "sum" | "average" => {
                        numbers.iter().try_fold(Decimal::ZERO, |total, value| {
                            checked_decimal_value(total.checked_add(*value), name)
                        })?
                    }
                    "min" => *numbers.iter().min().expect("non-empty arguments"),
                    "max" => *numbers.iter().max().expect("non-empty arguments"),
                    _ => unreachable!(),
                };
                if name == "average" {
                    checked_decimal(result.checked_div(Decimal::from(numbers.len())), "AVERAGE")
                } else {
                    Ok(Value::Number(result))
                }
            }
            "count" => {
                require_at_least(name, args, 1)?;
                Ok(Value::Number(Decimal::from(
                    values.iter().filter(|value| value.is_number()).count(),
                )))
            }
            "pv" | "fv" | "pmt" => evaluate_financial(name, args, &values),
            "npv" => evaluate_npv(args, &values),
            "simple_interest" | "compound_interest" => evaluate_interest(name, args, &values),
            "trim" | "upper" | "lower" | "length" => {
                require_arity(name, args, 1)?;
                let value = string(&values[0], name)?;
                match name {
                    "trim" => Ok(Value::String(value.trim().to_string())),
                    "upper" => Ok(Value::String(value.to_uppercase())),
                    "lower" => Ok(Value::String(value.to_lowercase())),
                    "length" => Ok(Value::Number(Decimal::from(value.chars().count()))),
                    _ => unreachable!(),
                }
            }
            "replace" => {
                require_arity(name, args, 4)?;
                let value = string(&values[0], "REPLACE")?;
                let start =
                    decimal_to_usize(number(&values[1], "REPLACE start")?, "REPLACE start")?;
                let length =
                    decimal_to_usize(number(&values[2], "REPLACE length")?, "REPLACE length")?;
                let replacement = string(&values[3], "REPLACE")?;
                let chars = value.chars().collect::<Vec<_>>();
                let start = start.min(chars.len());
                let end = start.saturating_add(length).min(chars.len());
                Ok(Value::String(
                    chars[..start].iter().collect::<String>()
                        + replacement
                        + &chars[end..].iter().collect::<String>(),
                ))
            }
            "left" | "right" => {
                require_arity(name, args, 2)?;
                let value = string(&values[0], name)?;
                let length = decimal_to_usize(
                    number(&values[1], &format!("{} length", name))?,
                    &format!("{} length", name),
                )?;
                let chars = value.chars().collect::<Vec<_>>();
                let length = length.min(chars.len());
                let selected = if name == "left" {
                    &chars[..length]
                } else {
                    &chars[chars.len() - length..]
                };
                Ok(Value::String(selected.iter().collect()))
            }
            "contains" => {
                require_arity(name, args, 2)?;
                Ok(Value::Bool(
                    string(&values[0], "CONTAINS")?.contains(string(&values[1], "CONTAINS")?),
                ))
            }
            "end_of_month" | "days_in_month" => {
                require_arity(name, args, 1)?;
                let date = parse_date(string(&values[0], name)?)?;
                let days = days_in_month(date.year(), date.month())?;
                if name == "days_in_month" {
                    Ok(Value::Number(Decimal::from(days)))
                } else {
                    let end = NaiveDate::from_ymd_opt(date.year(), date.month(), days)
                        .expect("validated calendar date")
                        .and_hms_opt(0, 0, 0)
                        .expect("valid midnight");
                    Ok(Value::String(format_date(end)))
                }
            }
            "add_months" => {
                require_arity(name, args, 2)?;
                let date = parse_date(string(&values[0], "ADD_MONTHS")?)?;
                let months = decimal_to_i64(
                    number(&values[1], "ADD_MONTHS months")?,
                    "ADD_MONTHS months",
                )?;
                let month_index = i64::from(date.year()) * 12 + i64::from(date.month0()) + months;
                let year = month_index.div_euclid(12);
                let month = month_index.rem_euclid(12) as u32 + 1;
                let year = i32::try_from(year).map_err(|_| {
                    CalculatorError::InvalidArgument(
                        "ADD_MONTHS result is outside the supported date range".to_string(),
                    )
                })?;
                let day = date.day().min(days_in_month(year, month)?);
                let result = NaiveDate::from_ymd_opt(year, month, day)
                    .and_then(|date_only| {
                        date_only.and_hms_opt(date.hour(), date.minute(), date.second())
                    })
                    .ok_or_else(|| {
                        CalculatorError::InvalidArgument(
                            "ADD_MONTHS result is outside the supported date range".to_string(),
                        )
                    })?;
                Ok(Value::String(format_date(result)))
            }
            "is_before" | "is_after" | "is_same_day" => {
                require_arity(name, args, 2)?;
                let left = parse_date(string(&values[0], name)?)?;
                let right = parse_date(string(&values[1], name)?)?;
                let result = match name {
                    "is_before" => left < right,
                    "is_after" => left > right,
                    "is_same_day" => left.date() == right.date(),
                    _ => unreachable!(),
                };
                Ok(Value::Bool(result))
            }
            _ => unreachable!("core built-in was not handled"),
        }
    }
}

fn expression_operation(expr: &Expr) -> &'static str {
    match expr {
        Expr::Number(_) => "number",
        Expr::String(_) => "string",
        Expr::Bool(_) => "boolean",
        Expr::Identifier(_) => "variable",
        Expr::Add(_, _) => "add",
        Expr::Subtract(_, _) => "subtract",
        Expr::Multiply(_, _) => "multiply",
        Expr::Divide(_, _) => "divide",
        Expr::Power(_, _) => "power",
        Expr::Modulo(_, _) => "modulo",
        Expr::Equal(_, _) => "equal",
        Expr::NotEqual(_, _) => "not_equal",
        Expr::LessThan(_, _) => "less_than",
        Expr::GreaterThan(_, _) => "greater_than",
        Expr::LessThanOrEqual(_, _) => "less_than_or_equal",
        Expr::GreaterThanOrEqual(_, _) => "greater_than_or_equal",
        Expr::And(_, _) => "and",
        Expr::Or(_, _) => "or",
        Expr::Not(_) => "not",
        Expr::UnaryMinus(_) => "unary_minus",
        Expr::FunctionCall { .. } => "function_call",
        Expr::Max(_, _) => "max",
        Expr::Min(_, _) => "min",
        Expr::Rnd(_, _) => "round",
        Expr::Ceil(_) => "ceil",
        Expr::Floor(_) => "floor",
        Expr::Exp(_) => "exp",
        Expr::Year(_) => "year",
        Expr::Month(_) => "month",
        Expr::Day(_) => "day",
        Expr::Substr(_, _, _) => "substr",
        Expr::AddDays(_, _) => "add_days",
        Expr::GetDiffDays(_, _) => "get_diff_days",
        Expr::PaddedString(_, _) => "padded_string",
        Expr::GetDiffMonths(_, _) => "get_diff_months",
        Expr::GetOutputFrom(_) => "get_output_from",
    }
}

fn is_core_builtin(name: &str) -> bool {
    matches!(
        name,
        "if" | "abs"
            | "sign"
            | "roundup"
            | "round_up"
            | "rounddown"
            | "round_down"
            | "trunc"
            | "clamp"
            | "pv"
            | "fv"
            | "pmt"
            | "npv"
            | "simple_interest"
            | "compound_interest"
            | "sum"
            | "average"
            | "count"
            | "min"
            | "max"
            | "trim"
            | "upper"
            | "lower"
            | "length"
            | "replace"
            | "left"
            | "right"
            | "contains"
            | "end_of_month"
            | "days_in_month"
            | "add_months"
            | "is_before"
            | "is_after"
            | "is_same_day"
    )
}

fn require_arity(name: &str, args: &[Expr], expected: usize) -> Result<()> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(CalculatorError::InvalidArgument(format!(
            "{} requires {} argument{}",
            name.to_uppercase(),
            expected,
            if expected == 1 { "" } else { "s" }
        )))
    }
}

fn require_at_least(name: &str, args: &[Expr], minimum: usize) -> Result<()> {
    if args.len() >= minimum {
        Ok(())
    } else {
        Err(CalculatorError::InvalidArgument(format!(
            "{} requires at least {} argument{}",
            name.to_uppercase(),
            minimum,
            if minimum == 1 { "" } else { "s" }
        )))
    }
}

fn number(value: &Value, function: &str) -> Result<Decimal> {
    value.as_number().ok_or_else(|| {
        CalculatorError::TypeError(format!("{} requires numeric arguments", function))
    })
}

fn string<'a>(value: &'a Value, function: &str) -> Result<&'a str> {
    value.as_string().ok_or_else(|| {
        CalculatorError::TypeError(format!("{} requires string arguments", function))
    })
}

fn numeric_values(values: &[Value], function: &str) -> Result<Vec<Decimal>> {
    values.iter().map(|value| number(value, function)).collect()
}

fn checked_decimal_value(value: Option<Decimal>, operation: &str) -> Result<Decimal> {
    value.ok_or_else(|| {
        CalculatorError::EvalError(format!(
            "{} produced a value outside the supported decimal range",
            operation
        ))
    })
}

fn validate_financial_inputs(
    name: &str,
    args: &[Expr],
    values: &[Value],
    minimum: usize,
    maximum: usize,
) -> Result<(Decimal, Decimal, Decimal)> {
    if args.len() < minimum || args.len() > maximum {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} requires {} to {} arguments",
            name.to_uppercase(),
            minimum,
            maximum
        )));
    }
    let rate = number(&values[0], name)?;
    let nper = number(&values[1], name)?;
    if rate <= -Decimal::ONE {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} rate must be greater than -1",
            name.to_uppercase()
        )));
    }
    if nper <= Decimal::ZERO {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} nper must be greater than zero",
            name.to_uppercase()
        )));
    }
    let payment_or_pv = number(&values[2], name)?;
    Ok((rate, nper, payment_or_pv))
}

fn financial_type(name: &str, values: &[Value], index: usize) -> Result<Decimal> {
    let value = if values.len() > index {
        number(&values[index], name)?
    } else {
        Decimal::ZERO
    };
    if value == Decimal::ZERO || value == Decimal::ONE {
        Ok(value)
    } else {
        Err(CalculatorError::InvalidArgument(format!(
            "{} type must be 0 or 1",
            name.to_uppercase()
        )))
    }
}

fn evaluate_financial(name: &str, args: &[Expr], values: &[Value]) -> Result<Value> {
    let (rate, nper, third) = validate_financial_inputs(name, args, values, 3, 5)?;
    let fourth = if values.len() > 3 {
        number(&values[3], name)?
    } else {
        Decimal::ZERO
    };
    let payment_type = financial_type(name, values, 4)?;

    let result = if rate.is_zero() {
        match name {
            "fv" => {
                -(fourth
                    .checked_add(checked_decimal_value(third.checked_mul(nper), name)?)
                    .ok_or_else(|| CalculatorError::EvalError("FV overflow".to_string()))?)
            }
            "pv" => {
                -(fourth
                    .checked_add(checked_decimal_value(third.checked_mul(nper), name)?)
                    .ok_or_else(|| CalculatorError::EvalError("PV overflow".to_string()))?)
            }
            "pmt" => -checked_decimal_value(
                third
                    .checked_add(fourth)
                    .ok_or_else(|| CalculatorError::EvalError("PMT overflow".to_string()))?
                    .checked_div(nper),
                "PMT",
            )?,
            _ => unreachable!(),
        }
    } else {
        let factor = checked_decimal_value(
            (Decimal::ONE.checked_add(rate).expect("rate > -1")).checked_powd(nper),
            name,
        )?;
        let annuity_numerator = checked_decimal_value(factor.checked_sub(Decimal::ONE), name)?;
        let annuity = checked_decimal_value(annuity_numerator.checked_div(rate), name)?;
        let due_factor = checked_decimal_value(
            Decimal::ONE.checked_add(checked_decimal_value(rate.checked_mul(payment_type), name)?),
            name,
        )?;
        match name {
            "fv" => {
                let pv_growth = checked_decimal_value(fourth.checked_mul(factor), name)?;
                let payment_growth = checked_decimal_value(
                    checked_decimal_value(third.checked_mul(due_factor), name)?
                        .checked_mul(annuity),
                    name,
                )?;
                -checked_decimal_value(pv_growth.checked_add(payment_growth), "FV")?
            }
            "pv" => {
                let payment_value = checked_decimal_value(
                    checked_decimal_value(third.checked_mul(due_factor), name)?
                        .checked_mul(annuity),
                    name,
                )?;
                -checked_decimal_value(
                    checked_decimal_value(fourth.checked_add(payment_value), name)?
                        .checked_div(factor),
                    "PV",
                )?
            }
            "pmt" => {
                let numerator = checked_decimal_value(
                    checked_decimal_value(third.checked_mul(factor), name)?.checked_add(fourth),
                    name,
                )?;
                let denominator = checked_decimal_value(due_factor.checked_mul(annuity), name)?;
                -checked_decimal_value(numerator.checked_div(denominator), "PMT")?
            }
            _ => unreachable!(),
        }
    };
    Ok(Value::Number(result))
}

fn evaluate_npv(args: &[Expr], values: &[Value]) -> Result<Value> {
    require_at_least("NPV", args, 2)?;
    let rate = number(&values[0], "NPV")?;
    if rate <= -Decimal::ONE {
        return Err(CalculatorError::InvalidArgument(
            "NPV rate must be greater than -1".to_string(),
        ));
    }
    let base = Decimal::ONE.checked_add(rate).expect("rate > -1");
    let mut result = Decimal::ZERO;
    for (index, value) in values[1..].iter().enumerate() {
        let factor = checked_decimal_value(base.checked_powd(Decimal::from(index + 1)), "NPV")?;
        let discounted = checked_decimal_value(number(value, "NPV")?.checked_div(factor), "NPV")?;
        result = checked_decimal_value(result.checked_add(discounted), "NPV")?;
    }
    Ok(Value::Number(result))
}

fn evaluate_interest(name: &str, args: &[Expr], values: &[Value]) -> Result<Value> {
    require_arity(name, args, 3)?;
    let principal = number(&values[0], name)?;
    let rate = number(&values[1], name)?;
    let periods = number(&values[2], name)?;
    if periods < Decimal::ZERO {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} periods must not be negative",
            name.to_uppercase()
        )));
    }
    let result = if name == "simple_interest" {
        checked_decimal_value(
            checked_decimal_value(principal.checked_mul(rate), name)?.checked_mul(periods),
            name,
        )?
    } else {
        if rate <= -Decimal::ONE {
            return Err(CalculatorError::InvalidArgument(
                "COMPOUND_INTEREST rate must be greater than -1".to_string(),
            ));
        }
        let factor = checked_decimal_value(
            Decimal::ONE
                .checked_add(rate)
                .expect("rate > -1")
                .checked_powd(periods),
            name,
        )?;
        checked_decimal_value(
            principal.checked_mul(checked_decimal_value(
                factor.checked_sub(Decimal::ONE),
                name,
            )?),
            name,
        )?
    };
    Ok(Value::Number(result))
}

fn days_in_month(year: i32, month: u32) -> Result<u32> {
    let first_of_next = if month == 12 {
        NaiveDate::from_ymd_opt(
            year.checked_add(1).ok_or_else(|| {
                CalculatorError::InvalidArgument("Date is outside the supported range".to_string())
            })?,
            1,
            1,
        )
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| {
        CalculatorError::InvalidArgument("Date is outside the supported range".to_string())
    })?;
    Ok((first_of_next - chrono::Duration::days(1)).day())
}

fn format_date(date: NaiveDateTime) -> String {
    date.format("%Y-%m-%dT%H:%M:%S").to_string()
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

fn checked_decimal(value: Option<Decimal>, operation: &str) -> Result<Value> {
    value.map(Value::Number).ok_or_else(|| {
        CalculatorError::EvalError(format!(
            "{} produced a value outside the supported decimal range",
            operation
        ))
    })
}

fn decimal_to_i64(value: Decimal, argument: &str) -> Result<i64> {
    if !value.fract().is_zero() {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} must be an integer",
            argument
        )));
    }

    value.to_i64().ok_or_else(|| {
        CalculatorError::InvalidArgument(format!("{} is outside the supported range", argument))
    })
}

fn decimal_to_usize(value: Decimal, argument: &str) -> Result<usize> {
    if value.is_sign_negative() {
        return Err(CalculatorError::InvalidArgument(format!(
            "{} must not be negative",
            argument
        )));
    }

    decimal_to_i64(value, argument).and_then(|value| {
        usize::try_from(value).map_err(|_| {
            CalculatorError::InvalidArgument(format!("{} is outside the supported range", argument))
        })
    })
}

fn decimal_to_decimal_places(value: Decimal, argument: &str) -> Result<u32> {
    let places = decimal_to_usize(value, argument)?;
    u32::try_from(places)
        .ok()
        .filter(|places| *places <= Decimal::MAX_SCALE)
        .ok_or_else(|| {
            CalculatorError::InvalidArgument(format!(
                "{} must be between 0 and {}",
                argument,
                Decimal::MAX_SCALE
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::Parser;
    use std::str::FromStr;

    fn decimal(value: &str) -> Decimal {
        Decimal::from_str(value).unwrap()
    }

    fn create_evaluator() -> Evaluator {
        Evaluator::new(
            VariableCache::new(),
            FormulaResultCache::new(),
            FunctionCache::new(),
        )
    }

    #[test]
    fn test_evaluate_number() {
        let mut parser = Parser::new("return 42").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(decimal("42")));
    }

    #[test]
    fn test_evaluate_addition() {
        let mut parser = Parser::new("return 2 + 3").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(decimal("5")));
    }

    #[test]
    fn test_evaluate_decimal_arithmetic_without_float_round_trip() {
        let mut parser = Parser::new("return 0.1 + 0.2").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        assert_eq!(
            evaluator.evaluate(&program).unwrap(),
            Value::Number(decimal("0.3"))
        );
    }

    #[test]
    fn test_evaluate_rejects_fractional_date_and_string_indexes() {
        let evaluator = create_evaluator();

        let mut parser = Parser::new("return substr('abc', 0.5, 1)").unwrap();
        let program = parser.parse().unwrap();
        assert!(matches!(
            evaluator.evaluate(&program),
            Err(CalculatorError::InvalidArgument(_))
        ));

        let mut parser = Parser::new("return add_days('2024-01-01', 1.5)").unwrap();
        let program = parser.parse().unwrap();
        assert!(matches!(
            evaluator.evaluate(&program),
            Err(CalculatorError::InvalidArgument(_))
        ));
    }

    #[test]
    fn test_evaluate_if_true() {
        let mut parser = Parser::new("if (5 > 3) then return 100 else return 200 end").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(decimal("100")));
    }

    #[test]
    fn test_evaluate_if_false() {
        let mut parser = Parser::new("if (3 > 5) then return 100 else return 200 end").unwrap();
        let program = parser.parse().unwrap();
        let evaluator = create_evaluator();

        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, Value::Number(decimal("200")));
    }

    #[test]
    fn test_logical_operators_short_circuit() {
        let evaluator = create_evaluator();

        let mut and_parser = Parser::new("return false and (1 / 0 = 0)").unwrap();
        let and_program = and_parser.parse().unwrap();
        assert_eq!(
            evaluator.evaluate(&and_program).unwrap(),
            Value::Bool(false)
        );

        let mut or_parser = Parser::new("return true or (1 / 0 = 0)").unwrap();
        let or_program = or_parser.parse().unwrap();
        assert_eq!(evaluator.evaluate(&or_program).unwrap(), Value::Bool(true));
    }

    fn evaluate(input: &str) -> Result<Value> {
        let mut parser = Parser::new(input)?;
        create_evaluator().evaluate(&parser.parse()?)
    }

    #[test]
    fn test_business_builtins_and_lazy_if() {
        assert_eq!(
            evaluate("return ABS(-3.5)").unwrap(),
            Value::Number(decimal("3.5"))
        );
        assert_eq!(
            evaluate("return sign(-2)").unwrap(),
            Value::Number(decimal("-1"))
        );
        assert_eq!(
            evaluate("return ROUNDUP(-1.21, 1)").unwrap(),
            Value::Number(decimal("-1.3"))
        );
        assert_eq!(
            evaluate("return round_down(-1.29, 1)").unwrap(),
            Value::Number(decimal("-1.2"))
        );
        assert_eq!(
            evaluate("return clamp(15, 0, 10)").unwrap(),
            Value::Number(decimal("10"))
        );
        assert_eq!(
            evaluate("return IF(true, 1, 1 / 0)").unwrap(),
            Value::Number(decimal("1"))
        );
    }

    #[test]
    fn test_financial_builtins_zero_rate_and_payment_type() {
        assert_eq!(
            evaluate("return FV(0, 3, -10, 100)").unwrap(),
            Value::Number(decimal("-70"))
        );
        assert_eq!(
            evaluate("return PV(0, 3, -10, 70)").unwrap(),
            Value::Number(decimal("-40"))
        );
        assert_eq!(
            evaluate("return PMT(0, 3, 100, -40)").unwrap(),
            Value::Number(decimal("-20"))
        );
        assert_eq!(
            evaluate("return fv(0.1, 2, -100, 0, 0)").unwrap(),
            Value::Number(decimal("210"))
        );
        assert_eq!(
            evaluate("return fv(0.1, 2, -100, 0, 1)").unwrap(),
            Value::Number(decimal("231"))
        );
        assert_eq!(
            evaluate("return NPV(0.1, 110, 121)").unwrap(),
            Value::Number(decimal("200"))
        );
        assert_eq!(
            evaluate("return simple_interest(100, 0.1, 2)").unwrap(),
            Value::Number(decimal("20"))
        );
        assert_eq!(
            evaluate("return compound_interest(100, 0.1, 2)").unwrap(),
            Value::Number(decimal("21"))
        );
    }

    #[test]
    fn test_financial_builtins_validate_arguments() {
        for input in [
            "return pv(-1, 2, 3)",
            "return fv(0.1, 0, 3)",
            "return pmt(0.1, 2, 3, 0, 2)",
        ] {
            assert!(matches!(
                evaluate(input),
                Err(CalculatorError::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn test_variadic_aggregates_and_count_type_rules() {
        assert_eq!(
            evaluate("return SUM(0.1, 0.2, 0.3)").unwrap(),
            Value::Number(decimal("0.6"))
        );
        assert_eq!(
            evaluate("return average(1, 2, 3)").unwrap(),
            Value::Number(decimal("2"))
        );
        assert_eq!(
            evaluate("return MIN(4, 2, 3)").unwrap(),
            Value::Number(decimal("2"))
        );
        assert_eq!(
            evaluate("return max(4)").unwrap(),
            Value::Number(decimal("4"))
        );
        assert_eq!(
            evaluate("return count(1, 'two', true, 3)").unwrap(),
            Value::Number(decimal("2"))
        );
        assert!(matches!(
            evaluate("return sum(1, 'two')"),
            Err(CalculatorError::TypeError(_))
        ));
        assert!(matches!(
            evaluate("return average()"),
            Err(CalculatorError::InvalidArgument(_))
        ));
    }

    #[test]
    fn test_text_builtins_use_unicode_character_indexes() {
        assert_eq!(
            evaluate("return trim('  hi \n')").unwrap(),
            Value::String("hi".to_string())
        );
        assert_eq!(
            evaluate("return UPPER('straße')").unwrap(),
            Value::String("STRASSE".to_string())
        );
        assert_eq!(
            evaluate("return lower('ÉCOLE')").unwrap(),
            Value::String("école".to_string())
        );
        assert_eq!(
            evaluate("return length('a💖')").unwrap(),
            Value::Number(decimal("2"))
        );
        assert_eq!(
            evaluate("return replace('a💖c', 1, 1, 'b')").unwrap(),
            Value::String("abc".to_string())
        );
        assert_eq!(
            evaluate("return left('💖ab', 2)").unwrap(),
            Value::String("💖a".to_string())
        );
        assert_eq!(
            evaluate("return right('💖ab', 2)").unwrap(),
            Value::String("ab".to_string())
        );
        assert_eq!(
            evaluate("return contains('a💖c', '💖')").unwrap(),
            Value::Bool(true)
        );
        assert!(matches!(
            evaluate("return left('abc', -1)"),
            Err(CalculatorError::InvalidArgument(_))
        ));
    }

    #[test]
    fn test_date_builtins_clamp_month_end_and_compare() {
        assert_eq!(
            evaluate("return add_months('2023-01-31', 1)").unwrap(),
            Value::String("2023-02-28T00:00:00".to_string())
        );
        assert_eq!(
            evaluate("return ADD_MONTHS('2024-01-31', 1)").unwrap(),
            Value::String("2024-02-29T00:00:00".to_string())
        );
        assert_eq!(
            evaluate("return end_of_month('2024-02-01')").unwrap(),
            Value::String("2024-02-29T00:00:00".to_string())
        );
        assert_eq!(
            evaluate("return days_in_month('2023-02-01')").unwrap(),
            Value::Number(decimal("28"))
        );
        assert_eq!(
            evaluate("return is_before('2024-01-01', '2024-01-02')").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            evaluate("return is_after('2024-01-02', '2024-01-01')").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            evaluate("return is_same_day('2024-01-01', '2024-01-01T23:00:00')").unwrap(),
            Value::Bool(true)
        );
    }
}
