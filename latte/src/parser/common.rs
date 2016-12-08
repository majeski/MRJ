use libc::*;
use std::ffi::CStr;

use ast::{Ident, Type};

use parser::to_ast::*;

impl ToAst<Ident> for *mut c_char {
    fn to_ast(&self) -> Ident {
        return Ident(self.to_ast());
    }
}

impl ToAst<String> for *mut c_char {
    fn to_ast(&self) -> String {
        let cstr = unsafe { CStr::from_ptr(*self) };
        let v = Vec::from(cstr.to_bytes());
        String::from_utf8(v).expect("asd")
    }
}

impl ToAst<Type> for *mut c_char {
    fn to_ast(&self) -> Type {
        let type_str: String = self.to_ast();
        match type_str.as_ref() {
            "int" => Type::TInt,
            "string" => Type::TString,
            "boolean" => Type::TBool,
            "void" => Type::TVoid,
            _ => panic!("unknown type"),
        }
    }
}
