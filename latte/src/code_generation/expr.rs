use ast::*;

use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;

impl GenerateCode<(Val, CGType)> for Expr {
    fn generate_code(&self, ctx: &mut Context) -> (Val, CGType) {
        let (reg, t) = match *self {
            Expr::EVar(ref ident) => {
                let (addr_reg, t) = ident.generate_code(ctx);
                let val = ctx.cg.add_load(addr_reg, t);
                if t == CGType::new(RawType::TString) {
                    ctx.cg.retain_string(Val::Reg(val));
                }
                (Val::Reg(val), t)
            }
            Expr::ELit(ref lit) => lit.generate_code(ctx),
            Expr::ECall(ref ident, ref args) => generate_call(ident, args, ctx),
            Expr::ENeg(ref e) => {
                let (val, t) = e.generate_code(ctx);
                (Val::Reg(ctx.cg.add_neg(val)), t)
            }
            Expr::ENot(ref e) => {
                let (val, t) = e.generate_code(ctx);
                (Val::Reg(ctx.cg.add_not(val)), t)
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
                    Operator::OpGreaterE => CGType::new(RawType::TBool),
                    Operator::OpAdd | Operator::OpSub | Operator::OpMul | Operator::OpDiv |
                    Operator::OpMod => CGType::new(RawType::TInt),
                    _ => unreachable!(),
                };
                (Val::Reg(ctx.cg.add_int_op(lhs_val, *op, rhs_val)), t)
            }
            Expr::ENew(ref t) => {
                let cgtype = ctx.to_cgtype(t);
                let obj = ctx.cg.new_object(cgtype);
                init_object(cgtype, obj, ctx);
                (Val::Reg(obj), cgtype)
            }
            Expr::ENewArray(ref t, ref size) => {
                let (size_val, _) = size.generate_code(ctx);
                let arr_t = CGType::new_arr(ctx.to_cgtype(t).t);
                let reg = ctx.cg.new_arr(arr_t, size_val);
                (Val::Reg(reg), arr_t)
            }
        };
        if t == CGType::new(RawType::TString) {
            ctx.add_string_tmp(reg);
        }
        (reg, t)
    }
}

fn generate_call(ident: &FieldGet, args: &Vec<Expr>, ctx: &mut Context) -> (Val, CGType) {
    let func_name: Ident = ident.generate_code(ctx);
    let ret_type = ctx.get_ret_type(&func_name);
    let arg_types = ctx.get_arg_types(&func_name);

    let mut arg_vals: Vec<(Val, CGType)> = Vec::new();
    for (arg_expr, arg_t) in args.iter().zip(arg_types) {
        let (mut arg_val, expr_t) = arg_expr.generate_code(ctx);
        if arg_t != expr_t {
            arg_val = Val::Reg(ctx.cg.bitcast_object(arg_val, expr_t, arg_t));
        }
        arg_vals.push((arg_val, arg_t));
    }

    (Val::Reg(ctx.cg.add_call(ret_type, &func_name.0, &arg_vals)), ret_type)
}

fn generate_or(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let lhs_label = ctx.cg.next_label();
    let rhs_label = ctx.cg.next_label();
    let end_label = ctx.cg.next_label();

    ctx.cg.add_jump(lhs_label);
    ctx.cg.add_label(lhs_label);
    let (lhs_val, _) = lhs.generate_code(ctx);
    let lhs_block = ctx.cg.get_current_label();
    ctx.cg.add_cond_jump(lhs_val, end_label, rhs_label);

    ctx.cg.add_label(rhs_label);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let rhs_block = ctx.cg.get_current_label();
    ctx.cg.add_jump(end_label);

    ctx.cg.add_label(end_label);
    let res_reg = ctx.cg.add_phi(CGType::new(RawType::TBool),
                                 (Val::Int(1), lhs_block),
                                 (rhs_val, rhs_block));
    (Val::Reg(res_reg), CGType::new(RawType::TBool))
}

fn generate_and(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let lhs_label = ctx.cg.next_label();
    let rhs_label = ctx.cg.next_label();
    let end_label = ctx.cg.next_label();

    ctx.cg.add_jump(lhs_label);
    ctx.cg.add_label(lhs_label);
    let (lhs_val, _) = lhs.generate_code(ctx);
    let lhs_block = ctx.cg.get_current_label();
    ctx.cg.add_cond_jump(lhs_val, rhs_label, end_label);

    ctx.cg.add_label(rhs_label);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let rhs_block = ctx.cg.get_current_label();
    ctx.cg.add_jump(end_label);

    ctx.cg.add_label(end_label);
    let res_reg = ctx.cg.add_phi(CGType::new(RawType::TBool),
                                 (Val::Int(0), lhs_block),
                                 (rhs_val, rhs_block));
    (Val::Reg(res_reg), CGType::new(RawType::TBool))
}

fn generate_add(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (lhs_val, t) = lhs.generate_code(ctx);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let result = match t.t {
        RawType::TInt => ctx.cg.add_int_op(lhs_val, Operator::OpAdd, rhs_val),
        RawType::TString => ctx.cg.add_add_str(lhs_val, rhs_val),
        _ => unreachable!(),
    };
    (Val::Reg(result), t)
}

fn generate_neq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (val, t) = generate_eq(lhs, rhs, ctx);
    (Val::Reg(ctx.cg.add_not(val)), t)
}

fn generate_eq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (lhs_val, t1) = lhs.generate_code(ctx);
    let (rhs_val, t2) = rhs.generate_code(ctx);
    let null_t = CGType::new(RawType::TNull);
    if t1 == null_t && t2 == null_t {
        return (Val::Int(1), CGType::new(RawType::TBool));
    }

    if let (RawType::TObject(lhs_id), RawType::TObject(rhs_id)) = (t1.t, t2.t) {
        return generate_objects_eq(lhs_val, lhs_id, rhs_val, rhs_id, ctx);
    }

    let t = if t1 != null_t { t1 } else { t2 };
    let result = ctx.cg.add_op(t, lhs_val, Operator::OpEq, rhs_val);
    (Val::Reg(result), CGType::new(RawType::TBool))
}

fn generate_objects_eq(mut lhs: Val, lhs_id: usize, mut rhs: Val, rhs_id: usize, ctx: &mut Context) -> (Val, CGType) {
    let lhs_t = CGType::new(RawType::TObject(lhs_id));
    let rhs_t = CGType::new(RawType::TObject(rhs_id));
    let mut t = lhs_t;
    if lhs_id != rhs_id {
        if ctx.is_subclass_of(lhs_id, rhs_id) {
            lhs = Val::Reg(ctx.cg.bitcast_object(lhs, lhs_t, rhs_t));
            t = rhs_t;
        } else if ctx.is_subclass_of(rhs_id, lhs_id) {
            rhs = Val::Reg(ctx.cg.bitcast_object(rhs, rhs_t, lhs_t));
            t = lhs_t;
        }
    }

    let result = ctx.cg.add_op(t, lhs, Operator::OpEq, rhs);
    (Val::Reg(result), CGType::new(RawType::TBool))
}

fn init_object(t: CGType, obj: Register, ctx: &mut Context) {
    let id = if let RawType::TObject(id) = t.t {
        id
    } else {
        unreachable!()
    };

    if let Some(super_id) = ctx.get_class_data(id).super_id {
        let super_t = CGType::new(RawType::TObject(super_id));
        let super_obj = ctx.cg.bitcast_object(Val::Reg(obj), t, super_t);
        init_object(super_t, super_obj, ctx);
    }

    let fields = ctx.get_class_data(id).get_fields();
    let str_t = CGType::new(RawType::TString);
    if !fields.iter().any(|f| ctx.get_class_data(id).get_field_type(f) == str_t) {
        return;
    }

    let empty_str = Val::Reg(ctx.cg.alloc_string());
    for field in fields {
        if ctx.get_class_data(id).get_field_type(&field) == str_t {
            let field_id = ctx.get_class_data(id).get_field_id(&field);
            let dst_addr = ctx.cg.get_field_addr(Val::Reg(obj), t, field_id);
            ctx.cg.add_store(dst_addr, str_t, empty_str);
            ctx.cg.retain_string(empty_str);
        }
    }
}

impl GenerateCode<(Val, CGType)> for Lit {
    fn generate_code(&self, ctx: &mut Context) -> (Val, CGType) {
        match *self {
            Lit::LInt(x) => (Val::Int(x), CGType::new(RawType::TInt)),
            Lit::LTrue => (Val::Int(1), CGType::new(RawType::TBool)),
            Lit::LFalse => (Val::Int(0), CGType::new(RawType::TBool)),
            Lit::LString(ref s) => {
                let reg = ctx.get_str_const(s);
                let val = ctx.cg.add_str_load(s.len(), reg);
                (Val::Reg(val), CGType::new(RawType::TString))
            }
            Lit::LNull(None) => (Val::Null, CGType::new(RawType::TNull)),
            Lit::LNull(Some(ref cname)) => {
                (Val::Null, ctx.to_cgtype(&Type::TObject(cname.clone())))
            }
        }
    }
}
