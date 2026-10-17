pub mod lexer;
pub mod parser;
pub mod interpreter;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args[1] != "run" {
        eprintln!("Usage: archarit run <file.arch>");
        std::process::exit(1);
    }

    let path = &args[2];
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("Cannot read {path}: {error}"));

    let tokens = lexer::tokenize(&source);
    let program = parser::parse(tokens);

    if let Err(error) = interpreter::run(program) {
        eprintln!("Runtime error: {error}");
        std::process::exit(1);
    }
}