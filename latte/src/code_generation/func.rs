use ast::Func;

use static_analysis::has_return::*;

use code_generation::context::Context;
use code_generation::generate::*;
use code_generation::code_generator::*;

impl GenerateCode<()> for Func {
    fn generate_code(&self, ctx: &mut Context) {
        let arg_types = self.args.iter().map(|a| CGType::from(&a.t)).collect();
        let ret_type = CGType::from(&self.ret_type);
        let arg_addr_regs = ctx.cg.add_func_begin(ret_type, &self.ident.0, &arg_types);
        for (arg, (arg_addr, t)) in self.args.iter().zip(arg_addr_regs) {
            ctx.set_var(arg.ident.clone(), arg_addr, t);
        }
        ctx.in_new_scope(|ctx| {
            self.body.generate_code(ctx);
            if !self.body.has_return() {
                ctx.release_all_strings();
            }
        });
        ctx.cg.add_func_end(ret_type);
    }
}
