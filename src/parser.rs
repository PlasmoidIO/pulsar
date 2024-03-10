use crate::{ast::Expression, token::Token};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn is(&self, token: Token) -> bool {
        self.tokens[self.current] == token
    }

    fn eat(&mut self, token: Token) -> bool {
        let eq = self.is(token);
        if eq {
            self.current += 1;
        }
        eq
    }
}
