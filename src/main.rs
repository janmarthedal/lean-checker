mod environment;
mod parser;

use parser::parse_lines;
use std::fs::File;

fn main() -> std::io::Result<()> {
    let file = File::open("examples/id.export")?;
    // let file = File::open("examples/Nat.gcd_self.export")?;

    match parse_lines(file) {
        Ok(_) => println!("Ok"),
        Err(_) => println!("Err"),
    }

    Ok(())
}
