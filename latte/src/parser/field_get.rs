use libc::*;

use ast::{Ident, FieldGet};

use parser::expr::*;
use parser::to_ast::*;

#[link(name = "parse", kind = "static")]
extern "C" {
    static FIELD_GET_TYPE_IDX: c_int;
    static FIELD_GET_TYPE_STD: c_int;
}

#[repr(C)]
pub struct field_get_t {
    t: i32,
    expr: *mut expr_t,
    ptr: *mut c_void,
}

impl ToAst<FieldGet> for field_get_t {
    fn to_ast(&self) -> TAResult<FieldGet> {
        unsafe {
            if self.t == FIELD_GET_TYPE_IDX {
                let e = self.expr.to_ast()?;
                let idx = (self.ptr as *mut expr_t).to_ast()?;
                return Ok(FieldGet::IdxAccess(Box::new(e), Box::new(idx)));
            }
            if self.t == FIELD_GET_TYPE_STD {
                let field: Ident = (self.ptr as *mut c_char).to_ast()?;
                return Ok(if self.expr.is_null() {
                    FieldGet::Direct(field)
                } else {
                    FieldGet::Indirect(Box::new(self.expr.to_ast()?), field)
                });
            }
            Err(format!("Unknown statement type: {}", self.t))
        }
    }
}
