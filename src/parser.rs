use crate::{
    ast::{Expression, Value},
    lexer::Lexer,
    token::Token,
};
use std::fmt::Display;

pub struct Parser {
    lexer: Lexer,
    lookahead: Token,
    line: usize,
    column: usize,
}

pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {} column {}: {}",
            self.line, self.column, self.message
        )
    }
}

macro_rules! eat {
    ($self:ident, $token:expr) => {
        if !$self.nibble($token) {
            return Err(ParseError {
                message: format!("expected {}", $token),
                line: $self.line,
                column: $self.column,
            });
        }
    };
}

impl Parser {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer::new(input);
        let lookahead = lexer.next_token();
        Self {
            lexer,
            lookahead,
            line: 1,
            column: 0,
        }
    }

    fn is(&self, token: Token) -> bool {
        self.lookahead == token
    }

    fn next_token(&mut self) -> Token {
        self.line = self.lexer.line;
        self.column = self.lexer.column;
        self.lexer.next_token()
    }

    fn nibble(&mut self, token: Token) -> bool {
        let eq = self.is(token);
        if eq {
            self.lookahead = self.next_token();
        }
        eq
    }

    pub fn expression(&mut self) -> Result<Expression, ParseError> {
        // TODO: implement statements (which are really just expressions...)
        self.assignment()
    }

    fn for_expression(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::For);

        let ident = match self.lookahead.clone() {
            Token::Ident(ident) => ident,
            _ => {
                return Err(ParseError {
                    message: "expected identifier".to_string(),
                    line: self.line,
                    column: self.column,
                })
            }
        };

        // checkpoint
    }

    fn assignment(&mut self) -> Result<Expression, ParseError> {
        let expr = self.equality()?;

        if self.nibble(Token::Eq) {
            let value = self.assignment();
            match expr {
                Expression::Identifier(ident) => {
                    return Ok(Expression::Assign {
                        name: ident,
                        value: Box::new(value?),
                    })
                }
                _ => {
                    return Err(ParseError {
                        message: "invalid assignment target".to_string(),
                        line: self.lexer.line,
                        column: self.lexer.column,
                    })
                }
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.comparison()?;

        while self.is(Token::EqEq) || self.is(Token::BangEq) {
            let op = self.lookahead.clone();
            self.lookahead = self.next_token();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(self.comparison()?),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.term()?;

        while self.is(Token::LessThan)
            || self.is(Token::LessThanEqual)
            || self.is(Token::GreaterThan)
            || self.is(Token::GreaterThanEqual)
        {
            let op = self.lookahead.clone();
            self.lookahead = self.next_token();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(self.term()?),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.factor()?;

        while self.is(Token::Plus) || self.is(Token::Minus) {
            let op = self.lookahead.clone();
            self.lookahead = self.next_token();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(self.factor()?),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.exponential()?;

        while self.is(Token::Asterisk) || self.is(Token::Slash) {
            let op = self.lookahead.clone();
            self.lookahead = self.next_token();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(self.exponential()?),
            };
        }

        Ok(expr)
    }

    fn exponential(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.unary()?;

        while self.nibble(Token::Pow) {
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: Token::Pow,
                right: Box::new(self.unary()?),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        self.call() // TODO
    }

    fn call(&mut self) -> Result<Expression, ParseError> {
        let callee = self.primary()?;

        // match self.lookahead {
        //     Token::Int(_) | Token::Float(_) | Token::Ident(_) => {
        //         return Ok(Expression::Call {
        //             function: Box::new(callee),
        //             arguments: vec![self.call()?],
        //         })
        //     }
        //     _ => {}
        // };

        if self.nibble(Token::LParen) {
            let mut args: Vec<Expression> = vec![];

            if !self.is(Token::RParen) {
                args.push(self.expression()?);
                while self.nibble(Token::Comma) {
                    args.push(self.expression()?);
                }
            }

            eat!(self, Token::RParen);
            return Ok(Expression::Call {
                function: Box::new(callee),
                arguments: args,
            });
        }

        Ok(callee)
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        if self.nibble(Token::LParen) {
            let expr = self.expression()?;
            eat!(self, Token::RParen);
            return Ok(expr);
        }

        if let Token::Ident(ident) = self.lookahead.clone() {
            self.lookahead = self.next_token();
            return Ok(Expression::Identifier(ident));
        }

        let tok = self.lookahead.clone();
        self.lookahead = self.next_token();
        match tok {
            Token::True => Ok(Expression::Value(Value::Boolean(true))),
            Token::False => Ok(Expression::Value(Value::Boolean(false))),
            Token::Int(i) => Ok(Expression::Value(Value::Int(i))),
            Token::Float(f) => Ok(Expression::Value(Value::Float(f))),
            Token::String(s) => Ok(Expression::Value(Value::String(s))),
            _ => Err(ParseError {
                message: format!("unexpected token: {:?}", tok),
                line: self.lexer.line,
                column: self.lexer.column,
            }),
        }
    }
}
