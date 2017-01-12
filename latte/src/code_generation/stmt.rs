use ast::*;

use static_analysis::has_return::*;

use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;

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
                let (addr_reg, t) = ident.generate_code(ctx);
                let (val_reg, _) = e.generate_code(ctx);
                ctx.cg.add_store(addr_reg, t, val_reg);
            }
            Stmt::SInc(ref ident) => {
                let (addr_reg, t) = ident.generate_code(ctx);
                let mut val_reg = ctx.cg.add_load(addr_reg, t);
                val_reg = ctx.cg.add_int_op(Val::Reg(val_reg), Operator::OpAdd, Val::Int(1));
                ctx.cg.add_store(addr_reg, t, Val::Reg(val_reg));
            }
            Stmt::SDec(ref ident) => {
                let (addr_reg, t) = ident.generate_code(ctx);
                let mut val_reg = ctx.cg.add_load(addr_reg, t);
                val_reg = ctx.cg.add_int_op(Val::Reg(val_reg), Operator::OpSub, Val::Int(1));
                ctx.cg.add_store(addr_reg, t, Val::Reg(val_reg));
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
            Stmt::SFor(_, ref ident, ref arr, ref stmt) => {
                let (arr, arr_t) = arr.generate_code(ctx);
                let before_loop = ctx.cg.next_label();
                let loop_begin = ctx.cg.next_label();
                let loop_body = ctx.cg.next_label();
                let loop_end = ctx.cg.next_label();
                let after_loop = ctx.cg.next_label();

                ctx.cg.add_jump(before_loop);
                ctx.cg.add_label(before_loop);
                let arr_size_ptr = ctx.cg.get_field_addr(arr, arr_t, 0);
                let arr_size = ctx.cg.add_load(arr_size_ptr, CGType::new(RawType::TInt));

                ctx.cg.add_jump(loop_begin);
                ctx.cg.add_label(loop_begin);
                let new_idx_reg = ctx.cg.next_reg();
                let idx_reg = ctx.cg.add_phi(CGType::new(RawType::TInt),
                                             (Val::Int(0), before_loop),
                                             (Val::Reg(new_idx_reg), loop_end));
                let valid_idx = ctx.cg
                    .add_int_op(Val::Reg(idx_reg), Operator::OpLess, Val::Reg(arr_size));
                ctx.cg.add_cond_jump(Val::Reg(valid_idx), loop_body, after_loop);

                ctx.cg.add_label(loop_body);
                ctx.in_new_scope(|mut ctx| {
                    let (elem_addr, elem_t) = ctx.cg
                        .get_nth_arr_elem(arr, arr_t, Val::Reg(idx_reg));
                    ctx.set_var(ident.clone(), elem_addr, elem_t);
                    stmt.generate_code(&mut ctx);
                });

                ctx.cg.add_jump(loop_end);
                ctx.cg.add_label(loop_end);
                ctx.cg.add_loop_step(new_idx_reg, idx_reg);
                ctx.cg.add_jump(loop_begin);

                ctx.cg.add_label(after_loop);
            }
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
                let t = CGType::from(t);
                let addr_reg = ctx.cg.add_alloca(t);
                let (val_reg, _) = e.generate_code(ctx);
                ctx.cg.add_store(addr_reg, t, val_reg);
                ctx.set_var(ident.clone(), addr_reg, t);
            }
            VarDecl::NoInit(ref t, ref ident) => {
                let default_lit = match *t {
                    Type::TInt => Lit::LInt(0),
                    Type::TBool => Lit::LFalse,
                    Type::TString => Lit::LString(String::new()),
                    Type::TObject(..) => Lit::LNull,
                    _ => unreachable!(),
                };
                let t = CGType::from(t);
                let addr_reg = ctx.cg.add_alloca(t);
                let (val_reg, _) = Expr::ELit(default_lit).generate_code(ctx);
                ctx.cg.add_store(addr_reg, t, val_reg);
                ctx.set_var(ident.clone(), addr_reg, t);
            }
        }
    }
}
