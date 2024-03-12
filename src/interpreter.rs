use std::{collections::HashMap, env, fmt::Display, mem::discriminant};

use crate::{ast::Expression, ast::Value, token::Token};

pub struct RuntimeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Runtime error at line {} column {}: {}",
            self.line, self.column, self.message
        )
    }
}

pub struct Interpreter {
    global_environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.insert(
            "print".to_string(),
            KoxValue::NativeFunction(NativeFunction {
                arity: 1,
                callable: |args| {
                    println!("{}", args[0]);
                    Ok(KoxValue::Nil)
                },
            }),
        );
        Self {
            global_environment: env,
        }
    }

    pub fn global_environment(&self) -> Environment {
        self.global_environment.clone()
    }

    fn evaluate_binary(
        &mut self,
        left: Expression,
        operator: Token,
        right: Expression,
        environment: &mut Environment,
        line: usize,
        column: usize,
    ) -> Result<KoxValue, RuntimeError> {
        let left = self.evaluate(left, environment)?;
        let right = self.evaluate(right, environment)?;

        if discriminant(&left) != discriminant(&right) {
            return Err(RuntimeError {
                message: "Operands must be of the same type".to_string(),
                line,
                column,
            });
        }

        match (left, right) {
            (KoxValue::Int(left), KoxValue::Int(right)) => match operator {
                Token::Plus => Ok(KoxValue::Int(left + right)),
                Token::Minus => Ok(KoxValue::Int(left - right)),
                Token::Asterisk => Ok(KoxValue::Int(left * right)),
                Token::Slash => Ok(KoxValue::Int(left / right)),
                Token::GreaterThan => Ok(KoxValue::Boolean(left > right)),
                Token::LessThan => Ok(KoxValue::Boolean(left < right)),
                Token::GreaterThanEqual => Ok(KoxValue::Boolean(left >= right)),
                Token::LessThanEqual => Ok(KoxValue::Boolean(left <= right)),
                _ => Err(RuntimeError {
                    message: format!("Invalid operator for integers: {:?}", operator),
                    line,
                    column,
                }),
            },
            (KoxValue::Float(left), KoxValue::Float(right)) => match operator {
                Token::Plus => Ok(KoxValue::Float(left + right)),
                Token::Minus => Ok(KoxValue::Float(left - right)),
                Token::Asterisk => Ok(KoxValue::Float(left * right)),
                Token::Slash => Ok(KoxValue::Float(left / right)),
                _ => Err(RuntimeError {
                    message: format!("Invalid operator for floats: {:?}", operator),
                    line,
                    column,
                }),
            },
            _ => Err(RuntimeError {
                message: format!("Invalid operands for operator: {:?}", operator),
                line,
                column,
            }),
        }
    }

    pub fn evaluate_program(
        &mut self,
        program: Vec<Expression>,
        env: &mut Environment,
    ) -> Result<KoxValue, RuntimeError> {
        let mut result = KoxValue::Nil;
        for expr in program {
            result = match self.evaluate(expr, env)? {
                KoxValue::Return(value) => return Ok(*value),
                value => value,
            };
        }
        Ok(result)
    }

    pub fn evaluate_expression(
        &mut self,
        expression: Expression,
    ) -> Result<KoxValue, RuntimeError> {
        self.evaluate(expression, &mut self.global_environment.clone())
    }

    fn evaluate(
        &mut self,
        expression: Expression,
        environment: &mut Environment,
    ) -> Result<KoxValue, RuntimeError> {
        match expression {
            Expression::Binary {
                left,
                operator,
                right,
                line,
                column,
            } => self.evaluate_binary(*left, operator, *right, environment, line, column),
            Expression::Call {
                function,
                arguments,
                line,
                column,
            } => {
                let callee = self.evaluate(*function, environment)?;

                let mut function: Box<dyn Callable> = match callee {
                    KoxValue::NativeFunction(function) => Box::new(function),
                    KoxValue::KoxFunction(function) => Box::new(function),
                    _ => {
                        return Err(RuntimeError {
                            message: format!("Can only call functions! Not {}", callee),
                            line,
                            column,
                        })
                    }
                };

                let mut args: Vec<KoxValue> = vec![];
                for arg in arguments {
                    args.push(self.evaluate(arg, environment)?);
                }

                if args.len() as u8 != function.arity() {
                    return Err(RuntimeError {
                        message: format!(
                            "Expected {} arguments but got {}",
                            function.arity(),
                            args.len()
                        ),
                        line,
                        column,
                    });
                }

                function.call(self, &args)
            }
            Expression::Identifier {
                ident,
                line,
                column,
            } => match environment.get(&ident) {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError {
                    message: format!("Undefined variable '{}'", ident),
                    line,
                    column,
                }),
            },
            Expression::Assign {
                name,
                value,
                line,
                column,
            } => {
                let value = self.evaluate(*value, environment)?;
                environment.assign(
                    Expression::Identifier {
                        ident: name.clone(),
                        line,
                        column,
                    },
                    value,
                )
            }
            Expression::Value { value, .. } => Ok(match value {
                Value::Int(i) => KoxValue::Int(i),
                Value::Float(f) => KoxValue::Float(f),
                Value::String(s) => KoxValue::String(s),
                Value::Boolean(b) => KoxValue::Boolean(b),
                Value::Nil => KoxValue::Nil,
            }),
            Expression::Let { name, value, .. } => {
                let value = self.evaluate(*value, environment)?;
                environment.insert(name, value.clone());
                Ok(value)
            }
            Expression::Return { value, .. } => {
                let result = self.evaluate(*value, environment)?;
                Ok(KoxValue::Return(Box::new(result)))
            }
            Expression::Block { expressions, .. } => {
                self.evaluate_program(expressions, &mut environment.child())
            }
            Expression::If {
                condition,
                consequence,
                alternative,
                line,
                column,
            } => {
                let condition = self.evaluate(*condition, environment)?;
                match condition {
                    KoxValue::Boolean(true) => self.evaluate(*consequence, environment),
                    KoxValue::Boolean(false) => match alternative {
                        Some(alt) => self.evaluate(*alt, environment),
                        None => Ok(KoxValue::Nil),
                    },
                    _ => Err(RuntimeError {
                        message: "Condition must be a boolean".to_string(),
                        line,
                        column,
                    }),
                }
            }
            Expression::Function {
                name,
                parameters,
                body,
                line,
                column,
            } => {
                let function = KoxFunction {
                    arity: parameters.len() as u8,
                    body: *body,
                    closure: environment.clone(),
                };
                environment.insert(name, KoxValue::KoxFunction(function));
                Ok(KoxValue::Nil)
            }
            Expression::For {
                ident,
                expr,
                body,
                line,
                column,
            } => todo!(),
        }
    }
}

trait Callable {
    fn arity(&self) -> u8;
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        args: &[KoxValue],
    ) -> Result<KoxValue, RuntimeError>;
}

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: u8,
    pub callable: fn(&[KoxValue]) -> Result<KoxValue, RuntimeError>,
}

impl Callable for NativeFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &mut self,
        _interpreter: &mut Interpreter,
        args: &[KoxValue],
    ) -> Result<KoxValue, RuntimeError> {
        (self.callable)(args)
    }
}

#[derive(Clone)]
pub struct KoxFunction {
    pub arity: u8,
    pub body: Expression,
    pub closure: Environment,
}

impl Callable for KoxFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        args: &[KoxValue],
    ) -> Result<KoxValue, RuntimeError> {
        let mut environment = self.closure.child();
        for (param, arg) in self.closure.venv.keys().zip(args) {
            environment.insert(param.to_string(), arg.clone());
        }
        interpreter.evaluate(self.body.clone(), &mut environment)
    }
}

#[derive(Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    venv: HashMap<String, KoxValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            venv: HashMap::new(),
        }
    }

    pub fn child(&mut self) -> Self {
        Self {
            enclosing: Some(Box::new(self.clone())),
            venv: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&KoxValue> {
        match self.venv.get(name) {
            Some(value) => Some(value),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.get(name),
                None => None,
            },
        }
    }

    pub fn insert(&mut self, name: String, value: KoxValue) {
        self.venv.insert(name, value);
    }

    pub fn assign(
        &mut self,
        identifier: Expression,
        value: KoxValue,
    ) -> Result<KoxValue, RuntimeError> {
        let (name, line, column) = match identifier.clone() {
            Expression::Identifier {
                ident,
                line,
                column,
            } => (ident, line, column),
            _ => panic!("attempting to assign to non-identifier expression"),
        };

        if self.venv.contains_key(&name) {
            self.venv.insert(name.to_string(), value.clone());
            return Ok(value);
        }

        match &mut self.enclosing {
            Some(enclosing) => enclosing.assign(identifier, value),
            None => Err(RuntimeError {
                message: format!("Undefined variable '{}'", name),
                line,
                column,
            }),
        }
    }
}

#[derive(Clone)]
pub enum KoxValue {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Nil,
    NativeFunction(NativeFunction),
    KoxFunction(KoxFunction),
    Return(Box<KoxValue>),
}

impl Display for KoxValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KoxValue::Int(i) => write!(f, "{}", i),
            KoxValue::Float(fl) => write!(f, "{}", fl),
            KoxValue::String(s) => write!(f, "{}", s),
            KoxValue::Boolean(b) => write!(f, "{}", b),
            KoxValue::Nil => write!(f, "nil"),
            KoxValue::NativeFunction(function) => write!(f, "<native function>"),
            KoxValue::KoxFunction(function) => write!(f, "<function>"),
            KoxValue::Return(value) => write!(f, "{}", value),
        }
    }
}
