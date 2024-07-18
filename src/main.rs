mod environment;
mod parser;

use parser::parse_lines;

fn process_file<R: std::io::Read>(file: R) -> Result<(), String> {
    match parse_lines(file) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let handle = std::io::stdin().lock();
        process_file(handle)
    } else if args.len() == 2 {
        match std::fs::File::open(&args[1]) {
            Ok(file) => process_file(file),
            Err(e) => Err(e.to_string())
        }
    } else {
        println!("Usage: lean-checker <export file path>");
        Ok(())
    }
}
