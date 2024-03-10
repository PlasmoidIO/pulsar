use std::env;

use parser::Parser;

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

    let mut parser = Parser::new(contents);
    let ast = parser.expression();
    match ast {
        Ok(ast) => println!("{:?}", ast),
        Err(e) => eprintln!("Error: {}", e),
    }
}
