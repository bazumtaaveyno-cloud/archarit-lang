use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("examples/demo.arch");

    match fs::read_to_string(path) {
        Ok(src) => {
            println!("Archarit Runner");
            println!("--- file: {} ---", path);
            println!("{}", &src.lines().take(15).collect::<Vec<_>>().join("\n"));
            println!("\n(parser stub) → parsed program placeholder");
        }
        Err(e) => {
            eprintln!("Failed to read {}: {}", path, e);
            std::process::exit(1);
        }
    }
}
