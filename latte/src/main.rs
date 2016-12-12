extern crate latte;

use latte::ast_print::*;
use latte::parser::run_parser;
use latte::types::*;

fn main() {
    match run_parser() {
        Ok(ref p) => {
            print_code(p);
            match check_types(p) {
                Err(e) => print!("{}", e),
                _ => ()
            };
        }
        Err(e) => println!("Parser error: {}", e),
    }
}
