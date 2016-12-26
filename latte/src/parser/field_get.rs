use libc::*;

use ast::FieldGet;

use parser::to_ast::*;

#[repr(C)]
pub struct field_get_t {
    ident: *mut c_char,
    field: *mut field_get_t,
}

impl ToAst<FieldGet> for field_get_t {
    fn to_ast(&self) -> TAResult<FieldGet> {
        Ok(FieldGet {
            ident: self.ident.to_ast()?,
            field: match self.field.is_null() {
                true => None,
                false => Some(Box::new(self.field.to_ast()?)),
            },
        })
    }
}
