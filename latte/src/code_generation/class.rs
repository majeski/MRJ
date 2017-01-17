use ast::Class;

use code_generation::generate::*;
use code_generation::context::Context;
use code_generation::cg_type::*;
use code_generation::code_generator::*;

impl GenerateCode<()> for Class {
    fn generate_code(&self, ctx: &mut Context) {
        ctx.in_new_scope(|ctx| {
            let id = ctx.get_class_id(&self.name);
            generate_new(id, ctx);
            generate_init(id, ctx);
            ctx.cg.reset();

            ctx.class = Some(id);
            for m in &self.methods {
                m.generate_code(ctx);
            }
            ctx.class = None;
        });
    }
}

fn generate_new(id: ClassId, ctx: &mut Context) {
    let t = CGType::obj_t(id);
    ctx.cg.add_func_begin(t, &format!("._new_{}", id), &vec![]);
    let obj = ctx.cg.new_object(t);
    ctx.cg.add_call(CGType::void_t(), format!("@._init_{}", id), &vec![(obj, t)]);
    ctx.cg.add_ret(t, obj);
    ctx.cg.add_func_end(t);
}

fn generate_init(id: ClassId, ctx: &mut Context) {
    let t = CGType::obj_t(id);
    let ret_type = CGType::void_t();
    let (obj_addr, _) = ctx.cg.add_func_begin(ret_type, &format!("._init_{}", id), &vec![t])[0];
    let obj = ctx.cg.add_load(obj_addr, t);
    if let Some(super_id) = ctx.get_class_data(id).super_id {
        let super_t = CGType::obj_t(super_id);
        let super_obj = ctx.cg.bitcast_object(obj, t, super_t);
        ctx.cg.add_call(ret_type,
                        format!("@._init_{}", super_id),
                        &vec![(super_obj, super_t)]);
    }

    {
        let vtable_size = ctx.get_class_data(id).vtable.size();
        ctx.cg.store_vtable(obj, id, vtable_size);
    }

    ctx.cg.add_comment(format!("Initialising strings"));
    init_strings(obj, id, ctx);
    ctx.cg.add_comment(format!("Initialising other fields"));
    init_vars(obj, id, ctx);

    ctx.cg.add_func_end(ret_type);
}

fn init_strings(obj: Val, id: ClassId, ctx: &mut Context) {
    let t = CGType::obj_t(id);
    let str_t = CGType::str_t();

    let fields = ctx.get_class_data(id).get_fields();
    if !fields.iter().any(|f| ctx.get_class_data(id).get_field_type(f) == str_t) {
        return;
    }

    let empty_str = ctx.cg.alloc_string();
    for field in fields {
        if ctx.get_class_data(id).get_field_type(&field) != str_t {
            continue;
        }
        let field_id = ctx.get_class_data(id).get_field_id(&field);
        let dst_addr = ctx.cg.get_field_addr(obj, t, field_id);
        ctx.cg.add_store(dst_addr, str_t, empty_str);
        ctx.cg.retain_string(empty_str);
    }
}

fn init_vars(obj: Val, id: ClassId, ctx: &mut Context) {
    let t = CGType::obj_t(id);
    let str_t = CGType::str_t();
    let fields = ctx.get_class_data(id).get_fields();

    for field in fields {
        let field_t = ctx.get_class_data(id).get_field_type(&field);
        if field_t.is_arr() || field_t == str_t {
            continue;
        }

        let field_id = ctx.get_class_data(id).get_field_id(&field);
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
}
