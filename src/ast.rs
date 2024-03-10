use crate::token::Token;
use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Identifier(String),
    Assign {
        name: String,
        value: Box<Expression>,
    },
    Value(Value),
    Let {
        name: String,
        value: Box<Expression>,
    },
    Return {
        value: Box<Expression>,
    },
    Block(Vec<Expression>),
    If {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Option<Box<Expression>>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Expression>,
    },
    For {
        ident: String,
        expr: Box<Expression>,
        body: Box<Expression>,
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
