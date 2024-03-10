use std::io::stdin;

use crate::{interpreter::Interpreter, parser::Parser};

pub fn repl() {
    let mut interpreter = Interpreter::new();

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let mut parser = Parser::new(input);
        let ast = parser.expression();
        match ast {
            Ok(ast) => match interpreter.evaluate_expression(ast) {
                Ok(value) => println!("{}", value),
                Err(e) => eprintln!("runtime error: {}", e),
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
