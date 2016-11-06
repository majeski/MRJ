use std::env;
use std::error::Error;
use std::io::{self, Read};
use std::process::Command;
use std::str;

extern crate instant;
use instant::ast::Program;
use instant::helpers::check_vars;
use instant::jvm;
use instant::llvm;
use instant::parser;


fn main() {
    let args: std::vec::Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./{} input_file", args[0]);
        return;
    }

    let path = std::path::Path::new(&args[1]);
    let raw_program = match read_file(path) {
        Err(why) => panic!(format!("Couldn't read file {}: {}", &args[1], why.description())),
        Ok(r) => r,
    };

    let program = parser::parse(raw_program).expect("Parse error");
    match check_vars(&program) {
        Err(why) => panic!(format!("Undefined variable: {}", why)),
        _ => {}
    };
    compile(program, &path).expect("Compilation error");
}

fn read_file(path: &std::path::Path) -> Result<String, io::Error> {
    let mut file = try!(std::fs::File::open(path));
    let mut out = String::new();
    try!(file.read_to_string(&mut out));
    Ok(out)
}

fn compile(program: Program, input: &std::path::Path) -> Result<(), io::Error> {
    if cfg!(feature = "jvm") {
        println!("generating jvm");
        try!(compile_jvm(program.clone(), input));
    }
    if cfg!(feature = "llvm") {
        println!("generating llvm");
        try!(compile_llvm(program, input));
    }
    Ok(())
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

    execute_bash_command(Command::new("java".to_string())
                             .arg("-jar")
                             .arg("lib/jasmin.jar".to_string())
                             .arg("-d")
                             .arg(out_class_dir.to_str().expect(err))
                             .arg(out_jasmin_path.to_str().expect(err)),
                         "Failed to call jasmin");
    Ok(())
}

fn compile_llvm(program: Program, input: &std::path::Path) -> Result<(), io::Error> {
    let err = "Something is wrong with file path";
    let filename = input.file_stem().expect(err).to_str().expect(err);
    let out_ll_path = input.with_file_name(filename.to_string() + ".ll");
    let out_bc_path_tmp = input.with_file_name(filename.to_string() + "_tmp.bc");
    let out_bc_path = input.with_file_name(filename.to_string() + ".bc");

    {
        let mut out_ll = std::fs::File::create(out_ll_path.clone()).unwrap();
        let mut ctx = llvm::LLVMContext::new(&mut out_ll);
        try!(llvm::compile(&program, &mut ctx));
    }

    // compile
    execute_bash_command(Command::new("llvm-as".to_string())
                             .arg(out_ll_path.to_str().expect(err))
                             .arg("-o")
                             .arg(out_bc_path_tmp.to_str().expect(err)),
                         "Failed to translate to LLVM bitcode");

    // link
    execute_bash_command(Command::new("llvm-link".to_string())
                             .arg("-o")
                             .arg(out_bc_path.to_str().expect(err))
                             .arg(out_bc_path_tmp.to_str().expect(err))
                             .arg("lib/runtime.bc".to_string()),
                         "Failed to link with runtime.bc");

    // rm temp .bc
    execute_bash_command(Command::new("rm".to_string()).arg(out_bc_path_tmp.to_str().expect(err)),
                         "Failed to clean up temporary .bc file");

    Ok(())
}

fn execute_bash_command(cmd: &mut Command, err: &'static str) {
    let es = cmd.status().expect(err);
    if !es.success() {
        panic!(err);
    }
}
