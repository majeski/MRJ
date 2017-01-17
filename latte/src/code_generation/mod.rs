use std::collections::HashMap;
use std::io::{self, Write};
use std::fs::File;

use ast::*;
use builtins::*;
use static_analysis::collect_string_lit::*;

mod cg_type;
mod class_data;
mod code_generator;
mod context;
mod expr;
mod field_get;
mod func;
mod generate;
mod stmt;
mod utils;

use self::cg_type::*;
use self::class_data::*;
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
            Def::DClass(ref c) => {
                ctx.in_new_scope(|ctx| {
                    let id = ctx.get_class_id(&c.name);
                    ctx.class = Some(id);
                    for m in &c.methods {
                        m.generate_code(ctx);
                    }
                    ctx.class = None;
                });
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
        if let Some(ref super_name) = class.superclass {
            class_data.set_super(*class_ids.get(super_name).unwrap());
        }

        for v in &class.vars {
            let t = match v.t {
                Type::TObject(ref cname) => {
                    let id = class_ids.get(cname).unwrap();
                    CGType::obj_t(*id)
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
        match *def {
            Def::DFunc(ref f) => {
                let ret_type = ctx.to_cgtype(&f.ret_type);
                let arg_types = f.args.iter().map(|arg| ctx.to_cgtype(&arg.t)).collect();
                ctx.add_func(&f.ident, arg_types, ret_type);
            }
            Def::DClass(ref c) => {
                let obj_t = ctx.to_cgtype(&Type::TObject(c.name.clone()));
                for f in &c.methods {
                    let ret_type = ctx.to_cgtype(&f.ret_type);
                    let mut arg_types: Vec<CGType> =
                        f.args.iter().map(|arg| ctx.to_cgtype(&arg.t)).collect();
                    arg_types.insert(0, obj_t);
                    let arg_types = arg_types;
                    ctx.add_func(&Ident(format!("class{}.{}", obj_t.get_id(), f.ident)),
                                 arg_types,
                                 ret_type);
                }
            }
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
