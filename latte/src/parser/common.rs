use libc::*;
use std::ffi::CStr;

use ast::{Ident, Type};

use parser::to_ast::*;

impl ToAst<Ident> for *mut c_char {
    fn to_ast(&self) -> TAResult<Ident> {
        return Ok(Ident(self.to_ast()?));
    }
}

impl ToAst<String> for *mut c_char {
    fn to_ast(&self) -> TAResult<String> {
        let cstr = unsafe { CStr::from_ptr(*self) };
        let v = Vec::from(cstr.to_bytes());
        match String::from_utf8(v) {
            Ok(x) => Ok(x),
            Err(x) => Err(format!("{}", x)),
        }
    }
}

impl ToAst<Type> for *mut c_char {
    fn to_ast(&self) -> TAResult<Type> {
        let type_str: String = self.to_ast()?;
        type_str.to_ast()
    }
}

impl ToAst<Type> for String {
    fn to_ast(&self) -> TAResult<Type> {
        if self.starts_with("[") {
            let inner: Type = String::from(self.split_at(1).1).to_ast()?;
            Ok(Type::TArray(Box::new(inner)))
        } else {
            let t = match self.as_ref() {
                "int" => Type::TInt,
                "string" => Type::TString,
                "boolean" => Type::TBool,
                "void" => Type::TVoid,
                _ => Type::TObject(Ident(self.clone())),
            };
            Ok(t)
        }
    }
}
