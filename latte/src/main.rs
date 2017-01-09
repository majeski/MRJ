use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, exit};

extern crate latte;

use latte::ast::Program;
use latte::code_generation;
use latte::optimization;
use latte::parser;
use latte::static_analysis;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

fn main() {
    match run() {
        Err(e) => {
            println_stderr!("ERROR\n{}", e);
            exit(-1);
        }
        Ok(..) => {
            println_stderr!("OK");
            exit(0);
        }
    }
}

fn run() -> Result<(), String> {
    let args: std::vec::Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(format!("Usage: ./{} input_file", args[0]));
    }

    let path = std::path::Path::new(&args[1]);
    let program_file = match File::open(path) {
        Err(why) => {
            return Err(format!("Couldn't open file {}: {}", &args[1], why));
        }
        Ok(r) => r,
    };

    let program = match parser::run(program_file) {
        Err(..) => {
            // error message printed from C code
            exit(-1);
        }
        Ok(x) => x,
    };

    match static_analysis::run(&program) {
        Err(why) => {
            return Err(format!("{}", why));
        }
        _ => {}
    };

    let program = optimization::run(program);

    match static_analysis::check_returns(&program) {
        Err(why) => {
            return Err(format!("{}", why));
        }
        _ => {}
    }

    match compile(&program, &path) {
        Err(why) => {
            return Err(format!("Compilation failed: {}", why));
        }
        _ => {}
    }

    Ok(())
}

fn compile(p: &Program, input: &std::path::Path) -> Result<(), io::Error> {
    let err = "Something is wrong with file path";
    let filename = input.file_stem().expect(err).to_str().expect(err);
    let out_ll_path = input.with_file_name(filename.to_string() + ".ll");
    let out_bc_path_tmp = input.with_file_name(filename.to_string() + "_tmp.bc");
    let out_bc_path = input.with_file_name(filename.to_string() + ".bc");
    {
        let mut out_ll = File::create(out_ll_path.clone())?;
        code_generation::gen_llvm(p, &mut out_ll)?;
    }

    // compile
    try!(execute_bash_command(Command::new("llvm-as")
                                  .arg(out_ll_path.to_str().expect(err))
                                  .arg("-o")
                                  .arg(out_bc_path_tmp.to_str().expect(err)),
                              "Failed to translate to LLVM bitcode"));

    // link
    try!(execute_bash_command(Command::new("llvm-link")
                                  .arg("-o")
                                  .arg(out_bc_path.to_str().expect(err))
                                  .arg(out_bc_path_tmp.to_str().expect(err))
                                  .arg("lib/runtime.bc"),
                              "Failed to link with runtime.bc"));

    // rm temp .bc
    try!(execute_bash_command(Command::new("rm").arg(out_bc_path_tmp.to_str().expect(err)),
                              "Failed to clean up temporary .bc file"));

    Ok(())
}

fn execute_bash_command(cmd: &mut Command, err: &'static str) -> Result<(), io::Error> {
    let es = try!(cmd.status());
    if !es.success() {
        Err(io::Error::new(io::ErrorKind::Other, err))
    } else {
        Ok(())
    }
}
