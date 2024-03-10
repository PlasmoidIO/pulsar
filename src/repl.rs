use std::io::stdin;

use crate::lexer::Lexer;

pub fn repl() {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        match Lexer::lex(input) {
            Ok(tokens) => println!("{:?}", tokens),
            Err(e) => eprintln!("{}", e),
        }
    }
}
