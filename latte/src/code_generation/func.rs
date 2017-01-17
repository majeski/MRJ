use ast::{Func, Ident};

use static_analysis::has_return::*;

use code_generation::cg_type::*;
use code_generation::context::Context;
use code_generation::generate::*;

impl GenerateCode<()> for Func {
    fn generate_code(&self, ctx: &mut Context) {
        let mut arg_types: Vec<CGType> = self.args.iter().map(|a| ctx.to_cgtype(&a.t)).collect();
        let mut arg_idents: Vec<Ident> = self.args.iter().map(|a| a.ident.clone()).collect();
        let mut name = self.ident.0.clone();
        if ctx.class.is_some() {
            arg_types.insert(0, CGType::obj_t(ctx.class.unwrap()));
            arg_idents.insert(0, Ident(format!("self")));
            name = format!("class{}.{}", ctx.class.unwrap(), name);
        }
        let arg_types = arg_types;
        let arg_idents = arg_idents;

        let ret_type = ctx.to_cgtype(&self.ret_type);
        let arg_addr_regs = ctx.cg.add_func_begin(ret_type, &name, &arg_types);
        for (ident, (arg_addr, t)) in arg_idents.into_iter().zip(arg_addr_regs) {
            ctx.set_var(ident, arg_addr, t);
        }
        ctx.in_new_scope(|ctx| {
            ctx.ret_type = ret_type;
            self.body.generate_code(ctx);
            if !self.body.has_return() {
                ctx.release_all_strings();
            }
        });
        ctx.cg.add_func_end(ret_type);
    }
}
