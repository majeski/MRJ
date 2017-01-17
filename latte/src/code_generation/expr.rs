use ast::*;

use code_generation::cg_type::*;
use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;

impl GenerateCode<(Val, CGType)> for Expr {
    fn generate_code(&self, ctx: &mut Context) -> (Val, CGType) {
        let (reg, t) = match *self {
            Expr::EVar(ref ident) => {
                let (addr_reg, t) = ident.generate_code(ctx);
                let reg = ctx.cg.add_load(addr_reg, t);
                if t == CGType::str_t() {
                    ctx.cg.retain_string(reg);
                }
                (reg, t)
            }
            Expr::ELit(ref lit) => lit.generate_code(ctx),
            Expr::ECall(ref ident, ref args) => generate_call(ident, args, ctx),
            Expr::ENeg(ref e) => {
                let (val, t) = e.generate_code(ctx);
                (ctx.cg.add_neg(val), t)
            }
            Expr::ENot(ref e) => {
                let (val, t) = e.generate_code(ctx);
                (ctx.cg.add_not(val), t)
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
                    Operator::OpGreaterE => CGType::bool_t(),
                    Operator::OpAdd | Operator::OpSub | Operator::OpMul | Operator::OpDiv |
                    Operator::OpMod => CGType::int_t(),
                    _ => unreachable!(),
                };
                (ctx.cg.add_int_op(lhs_val, *op, rhs_val), t)
            }
            Expr::ENew(ref t) => {
                let cgtype = ctx.to_cgtype(t);
                let obj = ctx.cg.new_object(cgtype);
                init_object(cgtype, obj, ctx);
                (obj, cgtype)
            }
            Expr::ENewArray(ref t, ref size) => {
                let (size_val, _) = size.generate_code(ctx);
                let arr_t = CGType::arr_t(ctx.to_cgtype(t).as_raw());
                let reg = ctx.cg.new_arr(arr_t, size_val);
                (reg, arr_t)
            }
        };
        if t == CGType::str_t() {
            ctx.add_string_tmp(reg);
        }
        (reg, t)
    }
}

fn generate_call(ident: &FieldGet, args: &Vec<Expr>, ctx: &mut Context) -> (Val, CGType) {
    let (obj, mut func_name): (Option<(Val, ClassId)>, Ident) = ident.generate_code(ctx);
    if obj.is_some() {
        let mut id = obj.unwrap().1;
        while !ctx.func_exists(&Ident(format!("class{}.{}", id, func_name))) {
            id = ctx.get_class_data(id).get_super();
        }
        func_name = Ident(format!("class{}.{}", id, func_name));
    }

    let ret_type = ctx.get_ret_type(&func_name);
    let arg_types = ctx.get_arg_types(&func_name);

    let mut arg_vals: Vec<(Val, CGType)> = args.iter().map(|a| a.generate_code(ctx)).collect();
    if let Some((val, id)) = obj {
        arg_vals.insert(0, (val, CGType::obj_t(id)));
    }

    let mut final_args: Vec<(Val, CGType)> = Vec::new();
    for ((mut arg_val, arg_t), arg_dst_t) in arg_vals.into_iter().zip(arg_types) {
        if arg_t != arg_dst_t {
            arg_val = ctx.cg.bitcast_object(arg_val, arg_t, arg_dst_t);
        }
        final_args.push((arg_val, arg_dst_t));
    }

    (ctx.cg.add_call(ret_type, &func_name.0, &final_args), ret_type)
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
    let res_reg = ctx.cg.add_phi(CGType::bool_t(),
                                 (Val::Int(1), lhs_block),
                                 (rhs_val, rhs_block));
    (res_reg, CGType::bool_t())
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
    let res_reg = ctx.cg.add_phi(CGType::bool_t(),
                                 (Val::Int(0), lhs_block),
                                 (rhs_val, rhs_block));
    (res_reg, CGType::bool_t())
}

fn generate_add(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (lhs_val, t) = lhs.generate_code(ctx);
    let (rhs_val, _) = rhs.generate_code(ctx);
    let val = match t.as_raw() {
        RawType::TInt => ctx.cg.add_int_op(lhs_val, Operator::OpAdd, rhs_val),
        RawType::TString => ctx.cg.concatenate_str(lhs_val, rhs_val),
        _ => unreachable!(),
    };
    (val, t)
}

fn generate_neq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (val, t) = generate_eq(lhs, rhs, ctx);
    (ctx.cg.add_not(val), t)
}

fn generate_eq(lhs: &Expr, rhs: &Expr, ctx: &mut Context) -> (Val, CGType) {
    let (lhs_val, t1) = lhs.generate_code(ctx);
    let (rhs_val, t2) = rhs.generate_code(ctx);
    let null_t = CGType::null_t();
    if t1 == null_t && t2 == null_t {
        return (Val::Int(1), CGType::bool_t());
    }

    if let (RawType::TObject(lhs_id), RawType::TObject(rhs_id)) = (t1.as_raw(), t2.as_raw()) {
        return generate_objects_eq(lhs_val, lhs_id, rhs_val, rhs_id, ctx);
    }

    let t = if t1 != null_t { t1 } else { t2 };
    let result = ctx.cg.add_op(t, lhs_val, Operator::OpEq, rhs_val);
    (result, CGType::bool_t())
}

fn generate_objects_eq(mut lhs: Val,
                       lhs_id: usize,
                       mut rhs: Val,
                       rhs_id: usize,
                       ctx: &mut Context)
                       -> (Val, CGType) {
    let lhs_t = CGType::obj_t(lhs_id);
    let rhs_t = CGType::obj_t(rhs_id);
    let mut t = lhs_t;
    if lhs_id != rhs_id {
        if ctx.is_subclass_of(lhs_id, rhs_id) {
            lhs = ctx.cg.bitcast_object(lhs, lhs_t, rhs_t);
            t = rhs_t;
        } else if ctx.is_subclass_of(rhs_id, lhs_id) {
            rhs = ctx.cg.bitcast_object(rhs, rhs_t, lhs_t);
            t = lhs_t;
        }
    }

    let result = ctx.cg.add_op(t, lhs, Operator::OpEq, rhs);
    (result, CGType::bool_t())
}

fn init_object(t: CGType, obj: Val, ctx: &mut Context) {
    let id = t.get_id();
    if let Some(super_id) = ctx.get_class_data(id).super_id {
        let super_t = CGType::obj_t(super_id);
        let super_obj = ctx.cg.bitcast_object(obj, t, super_t);
        init_object(super_t, super_obj, ctx);
    }

    let fields = ctx.get_class_data(id).get_fields();
    for field in &fields {
        let field_t = ctx.get_class_data(id).get_field_type(field);
        let field_id = ctx.get_class_data(id).get_field_id(&field);
        if field_t.is_arr() || field_t == CGType::str_t() {
            continue;
        }

        let dst_addr = ctx.cg.get_field_addr(obj, t, field_id);
        let val = if field_t.is_obj() {
            Val::Null
        } else if field_t == CGType::int_t() {
            Val::Int(0)
        } else if field_t == CGType::bool_t() {
            Val::Int(0)
        } else {
            unreachable!()
        };
        ctx.cg.add_store(dst_addr, field_t, val);
    }

    let str_t = CGType::str_t();
    if !fields.iter().any(|f| ctx.get_class_data(id).get_field_type(f) == str_t) {
        return;
    }

    let empty_str = ctx.cg.alloc_string();
    for field in fields {
        if ctx.get_class_data(id).get_field_type(&field) == str_t {
            let field_id = ctx.get_class_data(id).get_field_id(&field);
            let dst_addr = ctx.cg.get_field_addr(obj, t, field_id);
            ctx.cg.add_store(dst_addr, str_t, empty_str);
            ctx.cg.retain_string(empty_str);
        }
    }
}

impl GenerateCode<(Val, CGType)> for Lit {
    fn generate_code(&self, ctx: &mut Context) -> (Val, CGType) {
        match *self {
            Lit::LInt(x) => (Val::Int(x), CGType::int_t()),
            Lit::LTrue => (Val::Int(1), CGType::bool_t()),
            Lit::LFalse => (Val::Int(0), CGType::bool_t()),
            Lit::LString(ref s) => {
                let reg = ctx.get_str_const(s);
                let val = ctx.cg.add_str_load(s.len(), reg);
                (val, CGType::str_t())
            }
            Lit::LNull(None) => (Val::Null, CGType::null_t()),
            Lit::LNull(Some(ref cname)) => {
                (Val::Null, ctx.to_cgtype(&Type::TObject(cname.clone())))
            }
        }
    }
}
