use super::ast::{Expr, Program, Statement};
use super::lexer::{Lexer, Token};
use crate::error::{CalculatorError, Result};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self {
            tokens,
            position: 0,
        })
    }

    pub fn parse(&mut self) -> Result<Program> {
        let statement = self.parse_block()?;
        self.expect_token(Token::Eof)?;
        Ok(Program { statement })
    }

    fn parse_block(&mut self) -> Result<Statement> {
        if self.check_token(&Token::If) {
            self.parse_if_statement()
        } else if self.check_token(&Token::Return) {
            self.advance();
            let expr = self.parse_expression()?;
            Ok(Statement::Return(expr))
        } else if self.check_token(&Token::Error) {
            self.advance();
            self.expect_token(Token::LeftParen)?;
            let expr = self.parse_expression()?;
            self.expect_token(Token::RightParen)?;
            Ok(Statement::Error(expr))
        } else {
            Err(CalculatorError::ParseError(
                "Expected block statement".to_string(),
            ))
        }
    }

    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.expect_token(Token::If)?;
        self.expect_token(Token::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        self.expect_token(Token::Then)?;
        let then_block = Box::new(self.parse_block()?);

        let mut else_ifs = Vec::new();
        while self.check_token(&Token::Else) {
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len() {
                if let Token::If = self.tokens[next_pos] {
                    self.advance(); // consume Else
                    self.advance(); // consume If
                    self.expect_token(Token::LeftParen)?;
                    let else_if_condition = self.parse_expression()?;
                    self.expect_token(Token::RightParen)?;
                    self.expect_token(Token::Then)?;
                    let else_if_block = self.parse_block()?;
                    else_ifs.push((else_if_condition, else_if_block));
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let else_block = if self.check_token(&Token::Else) {
            self.advance();
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };

        self.expect_token(Token::End)?;

        Ok(Statement::If {
            condition,
            then_block,
            else_ifs,
            else_block,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;

        while self.check_token(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;

        while self.check_token(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;

        loop {
            if self.check_token(&Token::Equal) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::Equal(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::NotEqual) {
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::NotEqual(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;

        loop {
            if self.check_token(&Token::LessThan) {
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::LessThan(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::GreaterThan) {
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::GreaterThan(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::LessThanOrEqual) {
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::LessThanOrEqual(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::GreaterThanOrEqual) {
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::GreaterThanOrEqual(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            if self.check_token(&Token::Plus) {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::Add(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::Minus) {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::Subtract(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_modulo()?;

        loop {
            if self.check_token(&Token::Multiply) {
                self.advance();
                let right = self.parse_modulo()?;
                left = Expr::Multiply(Box::new(left), Box::new(right));
            } else if self.check_token(&Token::Divide) {
                self.advance();
                let right = self.parse_modulo()?;
                left = Expr::Divide(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_modulo(&mut self) -> Result<Expr> {
        let mut left = self.parse_power()?;

        while self.check_token(&Token::Mod) {
            self.advance();
            let right = self.parse_power()?;
            left = Expr::Modulo(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_power(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;

        if self.check_token(&Token::Power) {
            self.advance();
            let right = self.parse_power()?; // Right associative
            left = Expr::Power(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.check_token(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::UnaryMinus(Box::new(expr)))
        } else if self.check_token(&Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Not(Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let current = self.current_token();

        match current {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            Token::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Bool(b))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if self.check_token(&Token::LeftParen) {
                    self.advance();
                    let args = self.parse_argument_list()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(Expr::FunctionCall { name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            // Built-in functions
            Token::Max => self.parse_binary_function(Expr::Max),
            Token::Min => self.parse_binary_function(Expr::Min),
            Token::Rnd => self.parse_binary_function(Expr::Rnd),
            Token::Ceil => self.parse_unary_function(Expr::Ceil),
            Token::Floor => self.parse_unary_function(Expr::Floor),
            Token::Exp => self.parse_unary_function(Expr::Exp),
            Token::Year => self.parse_unary_function(Expr::Year),
            Token::Month => self.parse_unary_function(Expr::Month),
            Token::Day => self.parse_unary_function(Expr::Day),
            Token::Substr => self.parse_ternary_function(Expr::Substr),
            Token::AddDays => self.parse_binary_function(Expr::AddDays),
            Token::GetDiffDays => self.parse_binary_function(Expr::GetDiffDays),
            Token::PaddedString => self.parse_binary_function(Expr::PaddedString),
            Token::DifferenceInMonths => self.parse_binary_function(Expr::DifferenceInMonths),
            Token::GetOutputFrom => self.parse_unary_function(Expr::GetOutputFrom),
            _ => Err(CalculatorError::ParseError(format!(
                "Unexpected token: {:?}",
                current
            ))),
        }
    }

    fn parse_unary_function<F>(&mut self, constructor: F) -> Result<Expr>
    where
        F: FnOnce(Box<Expr>) -> Expr,
    {
        self.advance();
        self.expect_token(Token::LeftParen)?;
        let arg = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        Ok(constructor(Box::new(arg)))
    }

    fn parse_binary_function<F>(&mut self, constructor: F) -> Result<Expr>
    where
        F: FnOnce(Box<Expr>, Box<Expr>) -> Expr,
    {
        self.advance();
        self.expect_token(Token::LeftParen)?;
        let arg1 = self.parse_expression()?;
        self.expect_token(Token::Comma)?;
        let arg2 = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        Ok(constructor(Box::new(arg1), Box::new(arg2)))
    }

    fn parse_ternary_function<F>(&mut self, constructor: F) -> Result<Expr>
    where
        F: FnOnce(Box<Expr>, Box<Expr>, Box<Expr>) -> Expr,
    {
        self.advance();
        self.expect_token(Token::LeftParen)?;
        let arg1 = self.parse_expression()?;
        self.expect_token(Token::Comma)?;
        let arg2 = self.parse_expression()?;
        self.expect_token(Token::Comma)?;
        let arg3 = self.parse_expression()?;
        self.expect_token(Token::RightParen)?;
        Ok(constructor(Box::new(arg1), Box::new(arg2), Box::new(arg3)))
    }

    fn parse_argument_list(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();

        if self.check_token(&Token::RightParen) {
            return Ok(args);
        }

        args.push(self.parse_expression()?);

        while self.check_token(&Token::Comma) {
            self.advance();
            args.push(self.parse_expression()?);
        }

        Ok(args)
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn check_token(&self, token: &Token) -> bool {
        if self.position >= self.tokens.len() {
            return false;
        }
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token)
    }

    fn expect_token(&mut self, token: Token) -> Result<()> {
        if self.check_token(&token) {
            self.advance();
            Ok(())
        } else {
            Err(CalculatorError::ParseError(format!(
                "Expected {:?}, found {:?}",
                token,
                self.current_token()
            )))
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_return() {
        let mut parser = Parser::new("return 42").unwrap();
        let program = parser.parse().unwrap();

        match program.statement {
            Statement::Return(Expr::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected return statement with number"),
        }
    }

    #[test]
    fn test_parse_addition() {
        let mut parser = Parser::new("return 2 + 3").unwrap();
        let program = parser.parse().unwrap();

        match program.statement {
            Statement::Return(Expr::Add(_, _)) => {}
            _ => panic!("Expected return statement with addition"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let mut parser = Parser::new("if (5 > 3) then return 100 end").unwrap();
        let program = parser.parse().unwrap();

        match program.statement {
            Statement::If { .. } => {}
            _ => panic!("Expected if statement"),
        }
    }
}
