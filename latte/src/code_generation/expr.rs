use ast::*;

use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;
use code_generation::utils::get_ident;

impl GenerateCode<(RegOrInt, CGType)> for Expr {
    fn generate_code(&self, ctx: &mut Context) -> (RegOrInt, CGType) {
        match *self {
            Expr::EVar(ref ident) => {
                let addr_reg = ctx.get_var(get_ident(ident));
                let val = ctx.cg.add_load(addr_reg);
                (val, addr_reg.t)
            }
            Expr::ELit(ref lit) => lit.generate_code(ctx),
            Expr::ECall(ref ident, ref args) => generate_call(ident, args, ctx),
            Expr::ENeg(ref e) => {
                let (val, _) = e.generate_code(ctx);
                (ctx.cg.add_neg(val), CGType::TInt)
            }
            Expr::ENot(ref e) => {
                let (val, _) = e.generate_code(ctx);
                (ctx.cg.add_not(val), CGType::TBool)
            }
            Expr::EBinOp(ref lhs, Operator::OpOr, ref rhs) => generate_or(lhs, rhs, ctx),
            Expr::EBinOp(ref lhs, Operator::OpAnd, ref rhs) => generate_and(lhs, rhs, ctx),
            Expr::EBinOp(ref lhs, Operator::OpNEq, ref rhs) => generate_neq(lhs, rhs, ctx),
            Expr::EBinOp(ref lhs, Operator::OpEq, ref rhs) => generate_eq(lhs, rhs, ctx),
            Expr::EBinOp(ref lhs, Operator::OpAdd, ref rhs) => generate_add(lhs, rhs, ctx),
            Expr::EBinOp(ref lhs, ref op, ref rhs) => {
                let (lhs_val, _) = lhs.generate_code(ctx);
                let (rhs_val, _) = rhs.generate_code(ctx);
                let t = match *op {
                    Operator::OpLess | Operator::OpLessE | Operator::OpGreater |
                    Operator::OpGreaterE => CGType::TBool,
                    Operator::OpAdd | Operator::OpSub | Operator::OpMul | Operator::OpDiv |
                    Operator::OpMod => CGType::TInt,
                    _ => unreachable!(),
                };
                (ctx.cg.add_int_op(lhs_val, *op, rhs_val), t)
            }
        }
    }
}

fn generate_call(ident: &FieldGet, args: &Vec<Expr>, ctx: &mut Context) -> (RegOrInt, CGType) {
    let func_name = get_ident(ident);
    let ret_type = ctx.get_ret_type(func_name);

    let mut arg_vals: Vec<(RegOrInt, CGType)> = Vec::new();
    for arg in args {
        arg_vals.push(arg.generate_code(ctx));
    }

    (ctx.cg.add_call(ret_type, &func_name.0, &arg_vals), ret_type)
}

fn generate_or(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (RegOrInt, CGType) {
    let lhs_label = ctx.cg.next_label();
    let rhs_label = ctx.cg.next_label();
    let end_label = ctx.cg.next_label();

    ctx.cg.add_jump(lhs_label);
    ctx.cg.add_label(lhs_label);
    let (lhs_val, _) = lhs.generate_code(ctx);
    ctx.cg.add_cond_jump(lhs_val, end_label, rhs_label);

    ctx.cg.add_label(rhs_label);
    let (rhs_val, _) = rhs.generate_code(ctx);
    ctx.cg.add_jump(end_label);

    ctx.cg.add_label(end_label);
    let res_reg = ctx.cg.add_phi(CGType::TBool,
                                 (RegOrInt::Int(1), lhs_label),
                                 (rhs_val, rhs_label));
    (res_reg, CGType::TBool)
}

fn generate_and(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (RegOrInt, CGType) {
    let lhs_label = ctx.cg.next_label();
    let rhs_label = ctx.cg.next_label();
    let end_label = ctx.cg.next_label();

    ctx.cg.add_jump(lhs_label);
    ctx.cg.add_label(lhs_label);
    let (lhs_val, _) = lhs.generate_code(ctx);
    ctx.cg.add_cond_jump(lhs_val, rhs_label, end_label);

    ctx.cg.add_label(rhs_label);
    let (rhs_val, _) = rhs.generate_code(ctx);
    ctx.cg.add_jump(end_label);

    ctx.cg.add_label(end_label);
    let res_reg = ctx.cg.add_phi(CGType::TBool,
                                 (RegOrInt::Int(0), lhs_label),
                                 (rhs_val, rhs_label));
    (res_reg, CGType::TBool)
}

fn generate_add(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (RegOrInt, CGType) {
    let (lhs_val, t) = lhs.generate_code(ctx);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let result = match t {
        CGType::TInt => ctx.cg.add_int_op(lhs_val, Operator::OpAdd, rhs_val),
        CGType::TString => ctx.cg.add_add_str(lhs_val, rhs_val),
        _ => unreachable!(),
    };
    (result, t)
}

fn generate_neq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (RegOrInt, CGType) {
    let (val, t) = generate_eq(lhs, rhs, ctx);
    (ctx.cg.add_neg(val), t)
}

fn generate_eq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (RegOrInt, CGType) {
    let (lhs_val, t) = lhs.generate_code(ctx);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let result = match t {
        CGType::TInt => ctx.cg.add_int_op(lhs_val, Operator::OpEq, rhs_val),
        CGType::TString => ctx.cg.add_op(CGType::TString, lhs_val, Operator::OpEq, rhs_val),
        // TODO
        _ => unreachable!(),
    };
    (result, CGType::TBool)
}

impl GenerateCode<(RegOrInt, CGType)> for Lit {
    fn generate_code(&self, ctx: &mut Context) -> (RegOrInt, CGType) {
        match *self {
            Lit::LInt(x) => (RegOrInt::Int(x), CGType::TInt),
            Lit::LTrue => (RegOrInt::Int(1), CGType::TBool),
            Lit::LFalse => (RegOrInt::Int(0), CGType::TBool),
            Lit::LString(ref s) => {
                let reg = ctx.get_str_const(s);
                (ctx.cg.add_str_load(s.len(), reg), CGType::TString)
            }
            Lit::LNull => unimplemented!(), // TODO
        }
    }
}
