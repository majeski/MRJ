use std::str;
use std::io::{self, Read, Write};
use std::env;
use std::error::Error;
use std::process::Command;

extern crate instant;
use instant::parser;
use instant::jvm;
use instant::ast::Program;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("Failed printing to stderr");
    } }
);

fn read_file(path: &std::path::Path) -> Result<String, io::Error> {
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

    let path = std::path::Path::new(&args[1]);
    let raw_program = match read_file(path) {
        Err(why) => {
            println_stderr!("Couldn't read file {}: {}", &args[1], why.description());
            return;
        }
        Ok(r) => r,
    };

    let compilation_res = match parser::parse(raw_program) {
        Ok(program) => compile(program, &path),
        Err(e) => {
            println_stderr!("{}", e);
            return
        }
    };
}

fn compile(program: Program, input: &std::path::Path) -> Result<(), io::Error> {
    compile_jvm(program, input)
}

fn compile_jvm(mut program: Program, input: &std::path::Path) -> Result<(), io::Error> {
    let err = "Something is wrong with file path";
    let filename = input.file_stem().expect(err).to_str().expect(err);
    let out_jasmin_path = input.with_file_name(filename.to_string() + ".j");
    let out_class_dir = input.parent().expect(err);

    {
        let mut out_jasmin = std::fs::File::create(&out_jasmin_path).unwrap();
        jvm::optimize(&mut program);
        try!(jvm::compile(&program, filename, &mut out_jasmin));
    }

    Command::new("jasmin".to_string())
        .arg("-d")
        .arg(out_class_dir.to_str().expect(err))
        .arg(out_jasmin_path.to_str().expect(err))
        .output()
        .expect("Failed to call jasmin");
    Ok(())
}
