extern crate libc;

use libc::*;

use ast::Program;

use self::def::*;
use self::many::*;
use self::to_ast::*;

mod common;
mod def;
mod expr;
mod many;
mod stmt;
mod to_ast;

#[link(name = "parse", kind = "static")]
extern "C" {
    fn parse() -> c_int;
    fn free_parsed_defs();

    static parsed_defs: *mut many_t;
}

pub fn run_parser() -> Result<Program, String> {
    if unsafe { parse() } != 0 {
        return Err(format!("error in C code"));
    }

    let defs = many_t::to_vec(unsafe { parsed_defs }, def_t::to_ast);
    unsafe {
        free_parsed_defs();
    }

    defs.map(Program)
}
