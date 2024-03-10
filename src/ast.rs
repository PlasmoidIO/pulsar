use crate::token::Token;

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
    Identifier {
        value: String,
    },
    Assign {
        name: String,
        value: Box<Expression>,
    },
    Value(Value),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Return {
        value: Expression,
    },
    Expression {
        value: Expression,
    },
    Block {
        statements: Vec<Statement>,
    },
    If {
        condition: Expression,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Statement>,
    },
}
