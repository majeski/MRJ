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
    let mut ctx = create_context(p);

    for lit in collect_string_lit(p) {
        let reg = ctx.cg.add_string_constant(&lit);
        ctx.set_str_const(lit, reg);
    }
    ctx.cg.add_empty_line();

    for def in &p.0 {
        match *def {
            Def::DClass(..) => {
                // unimplemented!() // TODO
            }
            Def::DFunc(ref f) => f.generate_code(&mut ctx),
        }
        ctx.cg.reset();
    }

    for line in ctx.cg.get_out() {
        writeln!(out_file, "{}", line)?;
    }
    Ok(())
}

fn create_context(p: &Program) -> Context {
    let mut ctx = Context::new();

    add_classes(p, &mut ctx);
    ctx.cg.add_empty_line();

    add_funcs(p, &mut ctx);

    ctx.cg.add_comment(format!("builtin functions"));
    add_builtins(&mut ctx);
    ctx.cg.add_empty_line();
    ctx
}

fn add_classes(p: &Program, ctx: &mut Context) {
    let mut classes: Vec<&Class> = Vec::new();
    for def in &p.0 {
        if let Def::DClass(ref c) = *def {
            classes.push(c);
        }
    }

    let mut class_ids: HashMap<Ident, usize> = HashMap::new();
    for class in &classes {
        let id = class_ids.len();
        class_ids.insert(class.name.clone(), id);
    }

    for class in &classes {
        let id = class_ids.get(&class.name).unwrap();
        let mut class_data = ClassData::new(*id, &class.name);
        for v in &class.vars {
            let t = match v.t {
                Type::TObject(ref cname) => {
                    let id = class_ids.get(cname).unwrap();
                    CGType::new(RawType::TObject(*id))
                }
                _ => CGType::from(&v.t),
            };
            class_data.add_field(&v.ident, t);
        }
        ctx.add_class(*id, class_data);
    }
}

fn add_funcs(p: &Program, ctx: &mut Context) {
    for def in &p.0 {
        if let Def::DFunc(ref f) = *def {
            let ret_type = ctx.to_cgtype(&f.ret_type);
            let arg_types = f.args.iter().map(|arg| ctx.to_cgtype(&arg.t)).collect();
            ctx.add_func(&f.ident, arg_types, ret_type);
        }
    }
}

fn add_builtins(ctx: &mut Context) {
    for f in get_builtin_functions() {
        let ret_type = ctx.to_cgtype(&f.ret_type);
        let arg_types = f.args.iter().map(|t| ctx.to_cgtype(&t)).collect();
        ctx.cg.add_func_declare(ret_type, &f.ident.0, &arg_types);
        ctx.add_func(&f.ident, arg_types, ret_type);
    }
}
