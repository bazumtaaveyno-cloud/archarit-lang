mod division1_core;
mod division2_runtime;
mod division3_safety;
mod division4_toolchain;
mod division5_ui;
mod division6_collision;
mod division7_animation;
mod division8_web;
mod division9_c3l;
mod division10_ai;
mod division11_docs;

use division1_core::{Interpreter, Value};
use division4_toolchain::Cli;
use division9_c3l::C3LScene;

use std::env;
use std::fs;

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let command = Cli::parse(arguments);

    match command {
        Cli::Run { file } => {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("Cannot read {file}: {error}"));

            let mut interpreter = Interpreter::new();
            interpreter.run_source(&source).unwrap_or_else(|error| {
                eprintln!("Archarit error: {error}");
                std::process::exit(1);
            });
        }

        Cli::BuildC3L { file, output } => {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("Cannot read {file}: {error}"));

            let mut interpreter = Interpreter::new();
            let scene = interpreter.build_c3l_scene(&source).unwrap_or_else(|error| {
                eprintln!("C3L error: {error}");
                std::process::exit(1);
            });

            let json = serde_json::to_string_pretty(&scene)
                .expect("C3L scene serialization failed");

            fs::write(output, json).expect("Could not write C3L output");
            println!("C3L scene written.");
        }

        Cli::Check { file } => {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("Cannot read {file}: {error}"));

            let mut interpreter = Interpreter::new();
            interpreter.check_source(&source).unwrap_or_else(|error| {
                eprintln!("Archarit error: {error}");
                std::process::exit(1);
            });

            println!("No errors found.");
        }

        Cli::Help => {
            division11_docs::print_help();
        }
    }
}