extern crate latte;

use latte::ast_print::*;
use latte::parser::run_parser;

fn main() {
    match run_parser() {
        Ok(p) => p.print0(),
        Err(e) => println!("Parser error: {}", e),
    }
}
