use std::env;

mod lexer;
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

    match lexer::Lexer::lex(contents) {
        Ok(tokens) => println!("{:?}", tokens),
        Err(e) => eprintln!("{}", e),
    }
}
