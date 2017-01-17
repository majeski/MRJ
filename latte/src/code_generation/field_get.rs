use ast::*;

use code_generation::cg_type::*;
use code_generation::code_generator::*;
use code_generation::context::*;
use code_generation::generate::*;

impl GenerateCode<(Val, CGType)> for FieldGet {
    fn generate_code(&self, ctx: &mut Context) -> (Val, CGType) {
        match *self {
            FieldGet::Direct(ref ident) => {
                if ctx.var_exists(ident) {
                    ctx.get_var(ident)
                } else {
                    self_access(ident).generate_code(ctx)
                }
            }
            FieldGet::IdxAccess(ref arr, ref idx) => {
                let (struct_ptr, arr_t) = arr.generate_code(ctx);
                let (idx_val, _) = idx.generate_code(ctx);
                ctx.cg.get_nth_arr_elem(struct_ptr, arr_t, idx_val)
            }
            FieldGet::Indirect(ref expr, ref field) => {
                let (mut struct_addr, mut struct_type) = expr.generate_code(ctx);
                if struct_type.is_arr() {
                    (ctx.cg.get_field_addr(struct_addr, struct_type, 0), CGType::int_t())
                } else {
                    let mut id = struct_type.get_id();
                    let mut new_struct_type = struct_type;
                    while !ctx.get_class_data(id).has_field(field) {
                        id = ctx.get_class_data(id).get_super();
                        new_struct_type = CGType::obj_t(id);
                    }
                    if new_struct_type != struct_type {
                        struct_addr = ctx.cg
                            .bitcast_object(struct_addr, struct_type, new_struct_type);
                        struct_type = new_struct_type;
                    }
                    let field_t = ctx.get_class_data(id).get_field_type(field);
                    let field_id = ctx.get_class_data(id).get_field_id(field);
                    (ctx.cg.get_field_addr(struct_addr, struct_type, field_id), field_t)
                }
            }
        }
    }
}

impl GenerateCode<(Option<(Val, usize)>, Ident)> for FieldGet {
    fn generate_code(&self, ctx: &mut Context) -> (Option<(Val, usize)>, Ident) {
        match *self {
            FieldGet::Direct(ref ident) => {
                if ctx.func_exists(ident) {
                    (None, ident.clone())
                } else {
                    self_access(ident).generate_code(ctx)
                }
            }
            FieldGet::Indirect(ref expr, ref field) => {
                let (val, obj_t) = expr.generate_code(ctx);
                (Some((val, obj_t.get_id())), field.clone())
            }
            _ => unreachable!(),
        }
    }
}

fn self_access(ident: &Ident) -> FieldGet {
    let self_fg = FieldGet::Direct(Ident(format!("self")));
    FieldGet::Indirect(Box::new(Expr::EVar(self_fg)), ident.clone())
}
