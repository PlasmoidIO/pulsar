use std::fmt::{Display, Error};

use crate::token::Token;

pub struct Lexer {
    position: usize,
    ch: char,
    input: String,
    line: usize,
    column: usize,
}

pub struct LexerErrorInfo {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Display for LexerErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Lexer error at line {} column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl Lexer {
    pub fn next_token(&mut self) -> Token {
        while self.ch.is_whitespace() {
            if self.ch == '\n' {
                self.line += 1;
                self.column = 0;
            }

            self.advance();
        }

        let ch = self.ch;

        let tok = match ch {
            '\0' => Token::Eof,
            '=' if self.match_next('=') => Token::EqEq,
            '=' => Token::Eq,
            '+' => Token::Plus,
            '-' if self.peek().is_digit(10) => self.read_number(),
            '-' => Token::Minus,
            '!' if self.match_next('=') => Token::BangEq,
            '!' => Token::Bang,
            '*' => Token::Asterisk,
            '/' if self.match_next('/') => self.skip_line(),
            '/' => Token::Slash,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '"' | '\'' => self.read_string(ch),
            _ if self.ch.is_alphabetic() => return self.read_identifier(),
            _ if self.ch.is_digit(10) => return self.read_number(),
            _ => return Token::Illegal(format!("unexpected character: {}", ch)),
        };

        self.advance();

        tok
    }

    pub fn new(input: String) -> Self {
        let ch = input.chars().nth(0).unwrap();
        Self {
            position: 0,
            ch,
            input,
            line: 1,
            column: 0,
        }
    }

    pub fn lex(input: String) -> Result<Vec<Token>, LexerErrorInfo> {
        let mut lexer = Lexer::new(input);
        let mut tokens: Vec<Token> = vec![];
        let mut token = lexer.next_token();
        while token != Token::Eof {
            if let Token::Illegal(err) = token {
                return Err(LexerErrorInfo {
                    line: lexer.line,
                    column: lexer.column,
                    message: err,
                });
            }

            tokens.push(token.clone());
            token = lexer.next_token();
        }

        Ok(tokens)
    }

    fn match_next(&mut self, expected: char) -> bool {
        let matched = self.peek() == expected;
        if matched {
            self.advance();
        }
        matched
    }

    fn read_number(&mut self) -> Token {
        let pos = self.position;

        if self.ch == '-' {
            self.advance();
        }

        while self.ch.is_digit(10) {
            self.advance();
        }

        if self.ch == '.' {
            self.advance();

            while self.ch.is_digit(10) {
                self.advance();
            }

            let num = &self.input[pos..self.position];
            Token::Float(num.parse().unwrap())
        } else {
            let num = &self.input[pos..self.position];
            Token::Int(num.parse().unwrap())
        }
    }

    fn read_identifier(&mut self) -> Token {
        let pos = self.position;

        while self.ch.is_alphanumeric() {
            self.advance();
        }

        let ident = &self.input[pos..self.position];

        match ident {
            "fn" => Token::Function,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            _ => Token::Ident(ident.to_string()),
        }
    }

    fn read_string(&mut self, quote: char) -> Token {
        self.advance();
        let pos = self.position;

        while self.ch != quote {
            self.advance();

            if self.ch == '\0' {
                return Token::Illegal(format!("unterminated string - expected {}", quote));
            }
        }

        Token::String(self.input[pos..self.position].to_string())
    }

    fn skip_line(&mut self) -> Token {
        while self.ch != '\n' && self.ch != '\0' {
            self.advance();
        }
        self.next_token()
    }

    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
        self.ch = self.read_char_at(self.position);
    }

    fn peek(&self) -> char {
        self.read_char_at(self.position + 1)
    }

    fn read_char_at(&self, position: usize) -> char {
        match self.input.chars().nth(position) {
            Some(c) => c,
            None => '\0',
        }
    }
}
