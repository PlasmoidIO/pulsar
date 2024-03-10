use std::env;

use parser::Parser;

use crate::lexer::Lexer;

mod ast;
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
            let ast_string: Vec<String> = ast.iter().map(|e| format!("{}", e)).collect();
            println!("{}", ast_string.join("\n"));
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
