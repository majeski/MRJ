use ast::*;

use static_analysis::has_return::*;

use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;
use code_generation::utils::get_ident;

impl GenerateCode<()> for Vec<Stmt> {
    fn generate_code(&self, ctx: &mut Context) {
        for stmt in self {
            stmt.generate_code(ctx);
            if stmt.has_return() {
                break;
            }
        }
    }
}

impl GenerateCode<()> for Stmt {
    fn generate_code(&self, ctx: &mut Context) {
        match *self {
            Stmt::SEmpty => {}
            Stmt::SBlock(ref stmts) => ctx.in_new_scope(|ctx| stmts.generate_code(ctx)),
            Stmt::SDecl(_, ref decls) => {
                decls.generate_code(ctx);
            }
            Stmt::SAssign(ref ident, ref e) => {
                let addr_reg = ctx.get_var(get_ident(ident));
                let (val_reg, _) = e.generate_code(ctx);
                ctx.cg.add_store(addr_reg, val_reg);
            }
            Stmt::SInc(ref ident) => {
                let addr_reg = ctx.get_var(get_ident(ident));
                let mut val_reg = ctx.cg.add_load(addr_reg);
                val_reg = ctx.cg.add_int_op(val_reg, Operator::OpAdd, RegOrInt::Int(1));
                ctx.cg.add_store(addr_reg, val_reg);
            }
            Stmt::SDec(ref ident) => {
                let addr_reg = ctx.get_var(get_ident(ident));
                let mut val_reg = ctx.cg.add_load(addr_reg);
                val_reg = ctx.cg.add_int_op(val_reg, Operator::OpSub, RegOrInt::Int(1));
                ctx.cg.add_store(addr_reg, val_reg);
            }
            Stmt::SReturnE(ref e) => {
                let (val, t) = e.generate_code(ctx);
                ctx.cg.add_ret(t, val);
            }
            Stmt::SReturn => ctx.cg.add_ret_void(),
            Stmt::SExpr(ref e) => {
                e.generate_code(ctx);
            }
            Stmt::SIf(ref cond, ref s) => {
                let if_label = ctx.cg.next_label();
                let end_label = ctx.cg.next_label();

                let (cond_val, _) = cond.generate_code(ctx);
                ctx.cg.add_cond_jump(cond_val, if_label, end_label);

                ctx.cg.add_label(if_label);
                ctx.in_new_scope(|ctx| s.generate_code(ctx));
                ctx.cg.add_jump(end_label);

                ctx.cg.add_label(end_label);
            }
            Stmt::SIfElse(ref cond, ref if_true, ref if_false) => {
                let if_label = ctx.cg.next_label();
                let else_label = ctx.cg.next_label();
                let end_label = ctx.cg.next_label();
                let has_return = self.has_return();

                let (cond_val, _) = cond.generate_code(ctx);
                ctx.cg.add_cond_jump(cond_val, if_label, else_label);

                ctx.cg.add_label(if_label);
                ctx.in_new_scope(|ctx| if_true.generate_code(ctx));
                if !has_return {
                    ctx.cg.add_jump(end_label);
                }

                ctx.cg.add_label(else_label);
                ctx.in_new_scope(|ctx| if_false.generate_code(ctx));
                if !has_return {
                    ctx.cg.add_jump(end_label);
                    ctx.cg.add_label(end_label);
                }
            }
            Stmt::SWhile(ref cond, ref s) => {
                let cond_label = ctx.cg.next_label();
                let body_label = ctx.cg.next_label();
                let end_label = ctx.cg.next_label();

                ctx.cg.add_jump(cond_label);
                ctx.cg.add_label(cond_label);
                let (cond_val, _) = cond.generate_code(ctx);
                ctx.cg.add_cond_jump(cond_val, body_label, end_label);

                ctx.cg.add_label(body_label);
                ctx.in_new_scope(|ctx| s.generate_code(ctx));
                ctx.cg.add_jump(cond_label);

                ctx.cg.add_label(end_label);
            }
            Stmt::SFor(..) => unimplemented!(), // TODO
        }
    }
}

impl GenerateCode<()> for Vec<VarDecl> {
    fn generate_code(&self, ctx: &mut Context) {
        for var_decl in self {
            var_decl.generate_code(ctx);
        }
    }
}

impl GenerateCode<()> for VarDecl {
    fn generate_code(&self, ctx: &mut Context) {
        match *self {
            VarDecl::Init(ref t, ref ident, ref e) => {
                let addr_reg = ctx.cg.add_alloca(CGType::from(t));
                let (val_reg, _) = e.generate_code(ctx);
                ctx.cg.add_store(addr_reg, val_reg);
                ctx.set_var(ident.clone(), addr_reg);
            }
            VarDecl::NoInit(ref t, ref ident) => {
                let default_lit = match *t {
                    Type::TInt => Lit::LInt(0),
                    Type::TBool => Lit::LFalse,
                    Type::TString => Lit::LString(String::new()),
                    Type::TObject(..) => Lit::LNull,
                    _ => unreachable!(),
                };
                let addr_reg = ctx.cg.add_alloca(CGType::from(t));
                let (val_reg, _) = Expr::ELit(default_lit).generate_code(ctx);
                ctx.cg.add_store(addr_reg, val_reg);
                ctx.set_var(ident.clone(), addr_reg);
            }
        }
    }
}
