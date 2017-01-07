use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

extern crate latte;

use latte::ast::Program;
use latte::ast_print::print_code;
use latte::code_generation;
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
    match compile(&program, &path) {
        Err(why) => {
            println!("Compilation failed: {}", why);
            false
        }
        _ => true,
    }
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
