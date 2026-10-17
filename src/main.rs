mod division1_core_lexer;
mod division1_core_parser;
mod division1_core_checker;
mod division1_core_runtime;

#[cfg(test)]
mod division1_core_tests;

use std::env;
use std::fs;
use std::process::ExitCode;

use division1_core_checker::Checker;
use division1_core_parser::Parser;
use division1_core_runtime::Runtime;

enum Command {
    Run(String),
    Check(String),
    Help,
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn execute() -> Result<(), String> {
    let command = parse_arguments()?;

    let (path, check_only) = match command {
        Command::Run(path) => (path, false),
        Command::Check(path) => (path, true),
        Command::Help => {
            print_help();
            return Ok(());
        }
    };

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("Cannot read '{path}': {error}"))?;

    // Parser::parse calls the lexer internally.
    let program = Parser::parse(&source)
        .map_err(|error| format!("{path}: {error}"))?;

    Checker::check(&program)
        .map_err(|error| format!("{path}: {error}"))?;

    if check_only {
        println!("Check passed: {path}");
        return Ok(());
    }

    Runtime::run(&program)
        .map_err(|error| format!("{path}: {error}"))?;

    Ok(())
}

fn parse_arguments() -> Result<Command, String> {
    let mut arguments = env::args().skip(1);

    let Some(first) = arguments.next() else {
        return Ok(Command::Help);
    };

    let command = match first.as_str() {
        "-h" | "--help" | "help" => Command::Help,

        "run" => {
            let path = arguments.next().ok_or_else(|| {
                "Missing source file.
Usage: archarit-lang run <file.arch>"
                    .to_string()
            })?;

            Command::Run(path)
        }

        "check" => {
            let path = arguments.next().ok_or_else(|| {
                "Missing source file.
Usage: archarit-lang check <file.arch>"
                    .to_string()
            })?;

            Command::Check(path)
        }

        // Also allow: cargo run -- examples/demo.arch
        path if path.ends_with(".arch") => {
            Command::Run(path.to_string())
        }

        other => {
            return Err(format!(
                "Unknown command: '{other}'
\
                 Use --help to see available commands."
            ));
        }
    };

    if let Some(extra) = arguments.next() {
        return Err(format!("Unexpected extra argument: '{extra}'"));
    }

    Ok(command)
}

fn print_help() {
    println!(
        "Archarit — Division 1 development runner

Usage:
  archarit-lang run <file.arch>
  archarit-lang check <file.arch>
  archarit-lang <file.arch>
  archarit-lang --help

Using Cargo:
  cargo run -- run examples/demo.arch
  cargo run -- check examples/demo.arch
  cargo test

Commands:
  run     Parse, type-check, and execute a source file
  check   Parse and type-check without executing
  help    Show this help

Native, web, mobile, and C3L build targets are not implemented
by this entry point."
    );
}