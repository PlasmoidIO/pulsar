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
                message: format!("expected {:?}", $token),
                line: $self.line,
                column: $self.column,
            });
        }
    };
}

macro_rules! eat_identifier {
    ($self:ident) => {
        match $self.lookahead.clone() {
            Token::Ident(ident) => {
                $self.lookahead = $self.next_token();
                ident
            }
            _ => {
                return Err(ParseError {
                    message: "expected identifier".to_string(),
                    line: $self.line,
                    column: $self.column,
                })
            }
        }
    };
}

macro_rules! push_program {
    ($self:ident, $program:ident) => {
        $program.push($self.expression()?);
        match $program.last().unwrap() {
            Expression::For { body, .. } => {
                if let Expression::Block(_) = **body {
                    continue;
                }
            }
            Expression::Function { body, .. } => {
                if let Expression::Block(_) = **body {
                    continue;
                }
            }
            Expression::If {
                consequence,
                alternative,
                ..
            } => match alternative {
                Some(alt) => {
                    if let Expression::Block(_) = **alt {
                        continue;
                    }
                }
                None => {
                    if let Expression::Block(_) = **consequence {
                        continue;
                    }
                }
            },
            _ => {}
        }
        eat!($self, Token::Semicolon);
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

    pub fn parse_program(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut program: Vec<Expression> = vec![];
        while !self.is(Token::Eof) {
            push_program!(self, program);
        }
        Ok(program)
    }

    pub fn expression(&mut self) -> Result<Expression, ParseError> {
        // TODO: implement statements (which are really just expressions...)
        match self.lookahead {
            Token::For => self.for_expression(),
            Token::LBrace => self.block(),
            Token::Let => self.let_expression(),
            Token::Function => self.function_expression(),
            Token::If => self.if_expression(),
            _ => self.assignment(),
        }
    }

    fn if_expression(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::If);
        let condition = self.expression()?;
        let consequence = self.expression();
        let alternative = if self.nibble(Token::Else) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        Ok(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence?),
            alternative,
        })
    }

    fn function_expression(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::Function);
        let name = eat_identifier!(self);
        eat!(self, Token::LParen);
        let mut parameters: Vec<String> = vec![];
        if !self.is(Token::RParen) {
            parameters.push(eat_identifier!(self));
            while self.nibble(Token::Comma) {
                parameters.push(eat_identifier!(self));
            }
        }
        eat!(self, Token::RParen);
        Ok(Expression::Function {
            name,
            parameters,
            body: Box::new(self.expression()?),
        })
    }

    fn let_expression(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::Let);
        let ident = eat_identifier!(self);
        eat!(self, Token::Eq);
        let value = self.expression()?;
        Ok(Expression::Let {
            name: ident,
            value: Box::new(value),
        })
    }

    fn block(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::LBrace);
        let mut statements: Vec<Expression> = vec![];
        while !self.is(Token::RBrace) {
            push_program!(self, statements);
        }
        eat!(self, Token::RBrace);
        Ok(Expression::Block(statements))
    }

    fn for_expression(&mut self) -> Result<Expression, ParseError> {
        eat!(self, Token::For);

        let ident = eat_identifier!(self);
        eat!(self, Token::In);
        let expr = self.expression()?;
        Ok(Expression::For {
            ident,
            expr: Box::new(expr),
            body: Box::new(self.block()?),
        })
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
        let ret = match tok {
            Token::True => Ok(Expression::Value(Value::Boolean(true))),
            Token::False => Ok(Expression::Value(Value::Boolean(false))),
            Token::Int(i) => Ok(Expression::Value(Value::Int(i))),
            Token::Float(f) => Ok(Expression::Value(Value::Float(f))),
            Token::String(s) => Ok(Expression::Value(Value::String(s))),
            Token::Illegal(error) => Err(ParseError {
                message: error,
                line: self.lexer.line,
                column: self.lexer.column,
            }),
            _ => Err(ParseError {
                message: format!("unexpected token: {:?}", tok),
                line: self.lexer.line,
                column: self.lexer.column,
            }),
        };
        self.lookahead = self.next_token();
        ret
    }
}
