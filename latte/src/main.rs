extern crate latte;

use latte::ast_print::*;
use latte::parser::run_parser;

fn main() {
    match run_parser() {
        Ok(ref p) => print_code(p),
        Err(e) => println!("Parser error: {}", e),
    }
}
