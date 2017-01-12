use std::collections::HashMap;
use std::io::{self, Write};
use std::fs::File;

use ast::*;
use builtins::*;
use static_analysis::collect_string_lit::*;

mod code_generator;
mod context;
mod expr;
mod field_get;
mod func;
mod generate;
mod stmt;
mod utils;

use self::code_generator::*;
use self::context::*;
use self::generate::*;

pub fn gen_llvm(p: &Program, out_file: &mut File) -> Result<(), io::Error> {
    let mut funcs: HashMap<Ident, CGType> = HashMap::new();
    for def in &p.0 {
        if let Def::DFunc(ref f) = *def {
            funcs.insert(f.ident.clone(), CGType::from(&f.ret_type));
        }
    }
    for builtin in get_builtin_functions() {
        funcs.insert(builtin.ident.clone(), CGType::from(&builtin.ret_type));
    }

    let mut ctx = Context::new(funcs);

    for builtin in get_builtin_functions() {
        ctx.cg.add_func_declare(CGType::from(&builtin.ret_type),
                                &builtin.ident.0,
                                &builtin.args.iter().map(|t| CGType::from(t)).collect());
    }

    for lit in collect_string_lit(p) {
        let reg = ctx.cg.add_string_constant(&lit);
        ctx.set_str_const(lit, reg);
    }

    for def in &p.0 {
        match *def {
            Def::DClass(..) => unimplemented!(), // TODO
            Def::DFunc(ref f) => f.generate_code(&mut ctx),
        }
        ctx.cg.reset();
    }

    for line in ctx.cg.get_out() {
        writeln!(out_file, "{}", line)?;
    }
    Ok(())
}
