use std::str;
use std::io::{self, BufWriter, Read, Write};
use std::env;
use std::error::Error;

extern crate nom;
use nom::IResult;

extern crate instant;
use instant::parser::*;
use instant::jvm;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("Failed printing to stderr");
    } }
);

fn read_file(filename: &String) -> Result<String, io::Error> {
    let path = std::path::Path::new(filename);
    let mut file = try!(std::fs::File::open(path));
    let mut out = String::new();
    try!(file.read_to_string(&mut out));

    Ok(out)
}

fn main() {

    let args: std::vec::Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./{} input_file", args[0]);
        return;
    }

    let data = match read_file(&args[1]) {
        Err(why) => {
            println_stderr!("{}", why.description());
            return;
        }
        Ok(out) => out,
    };

    println_stderr!("{:?}", program(data.as_bytes()));

    if let IResult::Done(y, ref mut program) = program(data.as_bytes()) {
        if y.len() != 0 {
            panic!("asd!");
        }

        jvm::optimize(program);
        let mut output: BufWriter<_> = BufWriter::new(io::stdout());
        jvm::compile(program, &mut output);
        println_stderr!("{}", program);
    } else {
        panic!("zle zle zle");
    }

}
