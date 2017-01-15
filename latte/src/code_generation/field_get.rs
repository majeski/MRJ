use ast::*;

use code_generation::code_generator::*;
use code_generation::generate::*;
use code_generation::context::*;

impl GenerateCode<(Register, CGType)> for FieldGet {
    fn generate_code(&self, ctx: &mut Context) -> (Register, CGType) {
        match *self {
            FieldGet::Direct(ref ident) => ctx.get_var(ident),
            FieldGet::IdxAccess(ref arr, ref idx) => {
                let (struct_ptr, arr_t) = arr.generate_code(ctx);
                let (idx_val, _) = idx.generate_code(ctx);
                ctx.cg.get_nth_arr_elem(struct_ptr, arr_t, idx_val)
            }
            FieldGet::Indirect(ref expr, ref field) => {
                let (struct_addr, struct_type) = expr.generate_code(ctx);
                if struct_type.is_arr {
                    (ctx.cg.get_field_addr(struct_addr, struct_type, 0), CGType::new(RawType::TInt))
                } else if let RawType::TObject(id) = struct_type.t {
                    let field_t = ctx.get_class_data(id).get_field_type(field);
                    let field_id = ctx.get_class_data(id).get_field_id(field);
                    (ctx.cg.get_field_addr(struct_addr, struct_type, field_id), field_t)
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl GenerateCode<Ident> for FieldGet {
    fn generate_code(&self, _: &mut Context) -> Ident {
        match *self {
            FieldGet::Direct(ref ident) => ident.clone(),
            _ => panic!("Cannot get identifier"),
        }
    }
}
