extern crate latte;

use latte::ast_print::print_code;
use latte::parser::run_parser;
use latte::static_analysis::analyse;

fn main() {
    match run_parser() {
        Ok(ref p) => {
            print_code(p);
            match analyse(p) {
                Err(e) => print!("{}", e),
                _ => (),
            };
        }
        Err(e) => println!("Parser error: {}", e),
    }
}
