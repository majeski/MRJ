use libc::*;
use std::ffi::CStr;

use ast::{Ident, Type};

use parser::to_ast::*;

impl ToAst<Ident> for *mut c_char {
    fn to_ast(&self) -> TAResult<Ident> {
        return Ok(Ident(try!(self.to_ast())));
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
        let type_str: String = try!(self.to_ast());
        match type_str.as_ref() {
            "int" => Ok(Type::TInt),
            "string" => Ok(Type::TString),
            "boolean" => Ok(Type::TBool),
            "void" => Ok(Type::TVoid),
            _ => Err(format!("Unknown type: {}", type_str)),
        }
    }
}
