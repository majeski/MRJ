use std::env;
use std::fs::File;
use std::io::Write;
use std::io;

extern crate latte;

use latte::ast_print::print_code;
use latte::parser;
use latte::static_analysis;

fn main() {
    let stderr_msg = match run() {
        true => "OK",
        false => "ERROR",
    };
    writeln!(&mut io::stderr(), "{}", stderr_msg).unwrap();
}

fn run() -> bool {
    let args: std::vec::Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./{} input_file", args[0]);
        return false;
    }

    let path = std::path::Path::new(&args[1]);
    let program_file = match File::open(path) {
        Err(why) => {
            println!("Couldn't open file {}: {}", &args[1], why);
            return false;
        }
        Ok(r) => r,
    };

    let program = match parser::run(program_file) {
        Err(..) => {
            return false;
        }
        Ok(x) => x,
    };

    match static_analysis::run(&program) {
        Err(why) => {
            print!("{}", why);
            return false;
        }
        _ => {}
    };

    print_code(&program);
    return true;
}
