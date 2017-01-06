extern crate libc;

use libc::*;
use std::fs::File;
use std::os::unix::io::AsRawFd;

use ast::Program;

use self::def::*;
use self::many::*;
use self::to_ast::*;

mod common;
mod def;
mod expr;
mod field_get;
mod many;
mod stmt;
mod to_ast;

#[link(name = "parse", kind = "static")]
extern "C" {
    fn parse(fd: c_int) -> c_int;
    fn free_parsed_defs();

    static parsed_defs: *mut many_t;
}

pub fn run(f: File) -> Result<Program, String> {
    if unsafe { parse(f.as_raw_fd()) } != 0 {
        return Err(format!("error in C code"));
    }

    let defs = many_t::to_vec(unsafe { parsed_defs }, def_t::to_ast);
    unsafe {
        free_parsed_defs();
    }

    defs.map(Program)
}
