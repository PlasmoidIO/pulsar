use std::env;

use interpreter::Interpreter;
use parser::Parser;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        repl::repl();
        return;
    }
    let filepath = &args[1];
    run_file(filepath);
}

fn run_file(filepath: &str) {
    let contents =
        std::fs::read_to_string(filepath).expect("Something went wrong reading the file");

    let mut parser = Parser::new(contents.clone());
    let ast = parser.parse_program();
    match ast {
        Ok(ast) => {
            // map Vec<Expression> to Vec<String>
            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate_program(ast, &mut interpreter.global_environment());
            match result {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("runtime error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
