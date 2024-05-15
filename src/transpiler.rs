use crate::parser::Parser;
use crate::ast::Expression;
use crate::token::Token;

pub struct Transpiler {
    pub parser: Parser
}

impl Transpiler {
    pub fn new(input: String) -> Self {
        let parser = Parser::new(input);
        Transpiler {
            parser,
        }
    }

    pub fn transpile(&mut self) -> String {
        let program: Vec<Expression> = match self.parser.parse_program() {
            Ok(program) => program,
            Err(err) => panic!("{}", err)
        };

        let mut output = String::new();
        for expression in program {
            output.push_str(&self.transpile_expression(expression));
        }

        return output;
    }

    fn transpile_operator(&mut self, operator: Token) -> String {
        (match operator {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Asterisk => "*",
            Token::Slash => "/",
            Token::EqEq => "==",
            Token::BangEq => "!=",
            Token::LessThan => "<",
            Token::GreaterThan => ">",
            Token::LessThanEqual => "<=",
            Token::GreaterThanEqual => ">=",
            _ => panic!("Invalid operator: {:?}", operator),
        }).to_string()
    }

    fn transpile_call(&mut self, function: Expression, arguments: Vec<Expression>) -> String {
        let mut output = format!("{}(", self.transpile_expression(function));
        for argument in arguments {
            output.push_str(&self.transpile_expression(argument));
        }
        output.push_str(")");
        return output;
    }

    fn transpile_assignment(&mut self, name: String, value: Expression) -> String {
        format!("{} = {}", name, self.transpile_expression(value))
    }

    fn transpile_block(&mut self, expressions: Vec<Expression>) -> String {
        let mut vec: Vec<String> = vec![];

        for expression in expressions {
            vec.push(self.transpile_expression(expression));
        }

        let transpiled = vec.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\n");
        format!("{{\n{}\n}}", transpiled)
    }

    fn transpile_expression(&mut self, expression: Expression) -> String {
        let output = match expression {
            Expression::Binary { left, operator, right, .. } => format!("{} {} {}", self.transpile_expression(*left),
                self.transpile_operator(operator), self.transpile_expression(*right)),
            Expression::Call { function, arguments, .. } => self.transpile_call(*function, arguments),
            Expression::Identifier { ident, .. } => ident,
            Expression::Assign { name, value, .. } => self.transpile_assignment(name, *value),
            Expression::Value { value, .. } => format!("{}", value),
            Expression::Let { name, value, .. } => format!("let {}", self.transpile_assignment(name, *value)),
            Expression::Return { value, .. } => format!("return {}", self.transpile_expression(*value)),
            Expression::Block { expressions, .. } => self.transpile_block(expressions),
            Expression::If { condition, consequence, alternative, .. } => todo!(),
            Expression::Function { name, parameters, body, .. } => todo!(),
            Expression::For { ident, expr, body, .. } => todo!(),
        };

        todo!()
    }
}
