use crate::token::Token;
use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
        line: usize,
        column: usize,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
        line: usize,
        column: usize,
    },
    Identifier {
        ident: String,
        line: usize,
        column: usize,
    },
    Assign {
        name: String,
        value: Box<Expression>,
        line: usize,
        column: usize,
    },
    Value {
        value: Value,
        line: usize,
        column: usize,
    },
    Let {
        name: String,
        value: Box<Expression>,
        line: usize,
        column: usize,
    },
    Return {
        value: Box<Expression>,
        line: usize,
        column: usize,
    },
    Block {
        expressions: Vec<Expression>,
        line: usize,
        column: usize,
    },
    If {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Option<Box<Expression>>,
        line: usize,
        column: usize,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Expression>,
        line: usize,
        column: usize,
    },
    For {
        ident: String,
        expr: Box<Expression>,
        body: Box<Expression>,
        line: usize,
        column: usize,
    },
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
