use std::io::stdin;

use crate::{lexer::Lexer, parser::Parser};

pub fn repl() {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let mut parser = Parser::new(input);
        let ast = parser.expression();
        match ast {
            Ok(ast) => println!("{:?}", ast),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
