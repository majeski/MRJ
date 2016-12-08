extern crate latte;

use latte::ast_print::*;
use latte::parser::run_parser;

fn main() {
    let p = run_parser();
    println!("{:?}", p);
    p.unwrap().print0();
}
