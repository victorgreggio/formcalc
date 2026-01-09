use crate::error::{CalculatorError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Bool(bool),
    Identifier(String),

    // Keywords
    If,
    Then,
    Else,
    End,
    Return,
    Or,
    And,
    Mod,

    // Built-in functions
    Max,
    Min,
    Rnd,
    Ceil,
    Floor,
    Exp,
    Year,
    Month,
    Day,
    Substr,
    Error,
    AddDays,
    GetDiffDays,
    PaddedString,
    DifferenceInMonths,
    GetOutputFrom,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Not,

    // Delimiters
    LeftParen,
    RightParen,
    Comma,

    // End of file
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace_and_comments();

            if self.position >= self.input.len() {
                break;
            }

            let token = self.next_token()?;
            if token != Token::Eof {
                tokens.push(token);
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        let ch = self.current_char();

        match ch {
            '0'..='9' => self.read_number(),
            '\'' => self.read_string(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                Ok(Token::Minus)
            }
            '*' => {
                self.advance();
                Ok(Token::Multiply)
            }
            '/' => {
                self.advance();
                Ok(Token::Divide)
            }
            '^' => {
                self.advance();
                Ok(Token::Power)
            }
            '=' => {
                self.advance();
                Ok(Token::Equal)
            }
            '!' => {
                self.advance();
                Ok(Token::Not)
            }
            '<' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Ok(Token::NotEqual)
                } else if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::LessThanOrEqual)
                } else {
                    Ok(Token::LessThan)
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::GreaterThanOrEqual)
                } else {
                    Ok(Token::GreaterThan)
                }
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            _ => Err(CalculatorError::ParseError(format!(
                "Unexpected character: {}",
                ch
            ))),
        }
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.position;

        while self.position < self.input.len() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        if self.position < self.input.len() && self.current_char() == '.' {
            self.advance();
            while self.position < self.input.len() && self.current_char().is_ascii_digit() {
                self.advance();
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        let num = num_str
            .parse::<f64>()
            .map_err(|e| CalculatorError::ParseError(format!("Invalid number: {}", e)))?;

        Ok(Token::Number(num))
    }

    fn read_string(&mut self) -> Result<Token> {
        self.advance(); // skip opening '
        let mut result = String::new();

        while self.position < self.input.len() && self.current_char() != '\'' {
            let ch = self.current_char();
            if ch == '\\' {
                self.advance();
                if self.position < self.input.len() {
                    result.push(self.current_char());
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        if self.position >= self.input.len() {
            return Err(CalculatorError::ParseError(
                "Unterminated string".to_string(),
            ));
        }

        self.advance(); // skip closing '
        Ok(Token::String(result))
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token> {
        let start = self.position;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text: String = self.input[start..self.position].iter().collect();
        let lower = text.to_lowercase();

        let token = match lower.as_str() {
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "end" => Token::End,
            "return" => Token::Return,
            "or" => Token::Or,
            "and" => Token::And,
            "mod" => Token::Mod,
            "max" => Token::Max,
            "min" => Token::Min,
            "rnd" => Token::Rnd,
            "ceil" => Token::Ceil,
            "floor" => Token::Floor,
            "exp" => Token::Exp,
            "year" => Token::Year,
            "month" => Token::Month,
            "day" => Token::Day,
            "substr" => Token::Substr,
            "error" => Token::Error,
            "add_days" => Token::AddDays,
            "get_diff_days" => Token::GetDiffDays,
            "padded_string" => Token::PaddedString,
            "difference_in_months" => Token::DifferenceInMonths,
            "get_output_from" => Token::GetOutputFrom,
            "true" | "false" => Token::Bool(lower == "true"),
            _ => Token::Identifier(text),
        };

        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();

            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek() == Some('/') {
                // Line comment
                while self.position < self.input.len() && self.current_char() != '\n' {
                    self.advance();
                }
            } else if ch == '/' && self.peek() == Some('*') {
                // Block comment
                self.advance();
                self.advance();
                while self.position < self.input.len() - 1 {
                    if self.current_char() == '*' && self.peek() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn current_char(&self) -> char {
        if self.position < self.input.len() {
            self.input[self.position]
        } else {
            '\0'
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("42 3.15");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(42.0));
        assert_eq!(tokens[1], Token::Number(3.15));
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new("'hello world'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }

    #[test]
    fn test_tokenize_keywords() {
        let mut lexer = Lexer::new("if then else end return");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Then);
        assert_eq!(tokens[2], Token::Else);
        assert_eq!(tokens[3], Token::End);
        assert_eq!(tokens[4], Token::Return);
    }

    #[test]
    fn test_tokenize_operators() {
        let mut lexer = Lexer::new("+ - * / ^ = <> < > <= >=");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Multiply);
        assert_eq!(tokens[3], Token::Divide);
        assert_eq!(tokens[4], Token::Power);
        assert_eq!(tokens[5], Token::Equal);
        assert_eq!(tokens[6], Token::NotEqual);
        assert_eq!(tokens[7], Token::LessThan);
        assert_eq!(tokens[8], Token::GreaterThan);
        assert_eq!(tokens[9], Token::LessThanOrEqual);
        assert_eq!(tokens[10], Token::GreaterThanOrEqual);
    }

    #[test]
    fn test_tokenize_expression() {
        let mut lexer = Lexer::new("return 2 + 2");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Return);
        assert_eq!(tokens[1], Token::Number(2.0));
        assert_eq!(tokens[2], Token::Plus);
        assert_eq!(tokens[3], Token::Number(2.0));
    }
}
